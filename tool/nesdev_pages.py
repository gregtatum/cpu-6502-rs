#!/usr/bin/env python3
"""
Utility CLI for working with the NESDev MediaWiki site.

This script can:
  * enumerate pages (titles + last touched timestamps)
  * search for pages with a given query
  * fetch the HTML or wikitext for a page

Results are cached on disk so repeated queries are fast/offline-friendly.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Iterable, List, Tuple

import requests


API_URL = "https://www.nesdev.org/w/api.php"
CLOUDFLARE_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_0) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/118.0.0.0 Safari/537.36"
    ),
    "Accept": "application/json",
    "Accept-Language": "en-US,en;q=0.9",
}
REPO_ROOT = Path(__file__).resolve().parents[1]
CACHE_DIR = REPO_ROOT / "assets" / "nesdev_cache"
MAX_BATCH_SIZE = 500  # MediaWiki limit for anonymous requests.


class WikiError(RuntimeError):
    """Raised when the MediaWiki API reports an error."""


def ensure_cache_dir() -> None:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)


def cache_key(params: Dict[str, str]) -> str:
    serialized = json.dumps(sorted(params.items()), separators=(",", ":"))
    return hashlib.sha1(serialized.encode("utf-8")).hexdigest() + ".json"


def log_http_error(resp: requests.Response) -> None:
    """Dump useful debug data for HTTP errors."""
    print(
        f"HTTP error {resp.status_code} for {resp.url}",
        file=sys.stderr,
    )
    print("Response headers:", file=sys.stderr)
    for key, value in resp.headers.items():
        print(f"  {key}: {value}", file=sys.stderr)
    content = resp.text
    if len(content) > 2000:
        content = content[:2000] + "\n...[truncated]..."
    print("Response body:", file=sys.stderr)
    print(content, file=sys.stderr)


def cached_request(params: Dict[str, str], refresh: bool = False) -> Dict:
    """Fetch JSON from the MediaWiki API with a simple on-disk cache."""
    ensure_cache_dir()
    key = cache_key(params)
    cache_path = CACHE_DIR / key

    if not refresh and cache_path.exists():
        with cache_path.open("r", encoding="utf-8") as cache_file:
            return json.load(cache_file)

    response = requests.get(API_URL, params=params, headers=CLOUDFLARE_HEADERS, timeout=10)
    try:
        response.raise_for_status()
    except requests.HTTPError:
        log_http_error(response)
        raise
    data = response.json()

    if "error" in data:
        raise WikiError(data["error"].get("info", "Unknown MediaWiki error"))

    with cache_path.open("w", encoding="utf-8") as cache_file:
        json.dump(data, cache_file)

    return data


def iter_pages(prefix: str | None, limit: int, batch_size: int, refresh: bool) -> Iterable[Dict]:
    """Yield page metadata using generator=allpages so we also get timestamps."""
    fetched = 0
    continue_token: Dict[str, str] | None = None

    while fetched < limit:
        remaining = limit - fetched
        gap_limit = min(batch_size, remaining)
        params: Dict[str, str] = {
            "action": "query",
            "format": "json",
            "prop": "info",
            "generator": "allpages",
            "gaplimit": str(gap_limit),
        }
        if prefix:
            params["gapprefix"] = prefix
        if continue_token:
            params.update(continue_token)

        data = cached_request(params, refresh=refresh)
        pages = data.get("query", {}).get("pages", {})
        # pages is a dict keyed by pageid; keep output stable by sorting by 'index'.
        sorted_pages = sorted(
            pages.values(), key=lambda page: page.get("index", page["pageid"])
        )

        for page in sorted_pages:
            yield {
                "pageid": page["pageid"],
                "title": page["title"],
                "touched": page.get("touched", "unknown"),
            }
            fetched += 1
            if fetched >= limit:
                break

        continue_token = data.get("continue")
        if not continue_token:
            break


def output_list(pages: Iterable[Dict], fmt: str) -> None:
    pages_list = list(pages)
    if fmt == "json":
        json.dump(pages_list, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return
    if fmt == "csv":
        writer = csv.writer(sys.stdout)
        writer.writerow(["title", "touched"])
        for entry in pages_list:
            writer.writerow([entry["title"], entry["touched"]])
        return

    for entry in pages_list:
        print(f"{entry['title']} | last touched {entry['touched']}")


def command_list(args: argparse.Namespace) -> None:
    pages = iter_pages(
        prefix=getattr(args, "prefix", None),
        limit=getattr(args, "limit", 50),
        batch_size=getattr(args, "batch_size", MAX_BATCH_SIZE),
        refresh=args.refresh,
    )
    output_list(pages, getattr(args, "output_format", "text"))


def command_search(args: argparse.Namespace) -> None:
    params = {
        "action": "query",
        "format": "json",
        "list": "search",
        "srsearch": args.term,
        "srlimit": str(args.limit),
    }
    data = cached_request(params, refresh=args.refresh)
    results = data.get("query", {}).get("search", [])

    if args.output_format == "json":
        json.dump(results, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return

    for result in results:
        snippet = result.get("snippet", "").replace("<span class=\"searchmatch\">", "").replace(
            "</span>", ""
        )
        print(f"{result['title']} ({result.get('size', '?')} bytes)")
        if snippet:
            print(f"  {snippet}")


def command_fetch(args: argparse.Namespace) -> None:
    prop = "wikitext" if args.content_format == "wikitext" else "text"
    params = {
        "action": "parse",
        "format": "json",
        "page": args.title,
        "prop": prop,
    }
    data = cached_request(params, refresh=args.refresh)
    parsed = data.get("parse")
    if not parsed:
        raise WikiError("No parse data returned; ensure the title exists.")

    if args.content_format == "wikitext":
        print(parsed.get("wikitext", {}).get("*", ""))
    else:
        print(parsed.get("text", {}).get("*", ""))


def title_to_slug(title: str) -> str:
    """Convert a wiki title into a filesystem-friendly slug."""
    return title.replace(" ", "_")


def default_output_dir() -> Path:
    return REPO_ROOT / "assets" / "nesdev"


def format_metadata_wikitext(metadata: Dict[str, object]) -> str:
    """Render metadata as a Page metadata template block."""
    lines = ["{{Page metadata"]
    for key, value in metadata.items():
        if value is None:
            continue
        lines.append(f" | {key} = {value}")
    lines.append("}}")
    return "\n".join(lines)


def fetch_wikitext(title: str, refresh: bool) -> Tuple[Dict, str]:
    params = {
        "action": "parse",
        "format": "json",
        "page": title,
        "prop": "wikitext",
    }
    data = cached_request(params, refresh=refresh)
    parsed = data.get("parse")
    if not parsed:
        raise WikiError("No parse data returned; ensure the title exists.")

    wikitext = parsed.get("wikitext", {}).get("*")
    if wikitext is None:
        raise WikiError("Page did not include wikitext content.")
    return parsed, wikitext


def resolve_repo_path(path_str: str | None, default: Path) -> Path:
    if not path_str:
        return default
    candidate = Path(path_str)
    if candidate.is_absolute():
        return candidate
    return REPO_ROOT / candidate


def command_save(args: argparse.Namespace) -> None:
    parsed, wikitext = fetch_wikitext(args.title, refresh=args.refresh)
    slug = title_to_slug(args.slug if args.slug else parsed.get("title", args.title))

    output_dir = resolve_repo_path(args.output_dir, default_output_dir())
    output_dir.mkdir(parents=True, exist_ok=True)

    if args.output_path:
        output_path = resolve_repo_path(args.output_path, output_dir / f"{slug}.wikitext")
    else:
        output_path = output_dir / f"{slug}.wikitext"

    metadata = {
        "title": parsed.get("title"),
        "author": args.author,
        "date": args.date or datetime.now(timezone.utc).date().isoformat(),
        "source": f"https://www.nesdev.org/wiki/{title_to_slug(parsed.get('title', args.title))}",
        "pageid": parsed.get("pageid"),
        "revid": parsed.get("revid"),
    }

    front_matter = format_metadata_wikitext(metadata)
    content = wikitext.rstrip("\n")
    document = f"{front_matter}\n\n{content}\n"

    with output_path.open("w", encoding="utf-8") as outfile:
        outfile.write(document)

    print(f"Wrote {output_path}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="NESDev MediaWiki helper utility.")
    parser.add_argument(
        "--refresh",
        action="store_true",
        help="Ignore the local cache and fetch fresh data from the wiki.",
    )
    subparsers = parser.add_subparsers(dest="command")

    list_parser = subparsers.add_parser("list", help="Enumerate wiki pages.")
    list_parser.add_argument("--prefix", help="Limit results to titles that start with this prefix.")
    list_parser.add_argument(
        "--limit",
        type=int,
        default=50,
        help="Maximum number of pages to return (default: 50).",
    )
    list_parser.add_argument(
        "--batch-size",
        type=int,
        default=MAX_BATCH_SIZE,
        help=f"Internal page fetch batch size (max {MAX_BATCH_SIZE}).",
    )
    list_parser.add_argument(
        "--output-format",
        choices=["text", "json", "csv"],
        default="text",
        help="Output format when listing pages.",
    )
    list_parser.set_defaults(func=command_list)

    search_parser = subparsers.add_parser("search", help="Search for pages.")
    search_parser.add_argument("term", help="Search query.")
    search_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="Maximum number of results to return (default: 20).",
    )
    search_parser.add_argument(
        "--output-format",
        choices=["text", "json"],
        default="text",
        help="Output format for search results.",
    )
    search_parser.set_defaults(func=command_search)

    fetch_parser = subparsers.add_parser("fetch", help="Fetch HTML or wikitext for a page.")
    fetch_parser.add_argument("title", help="Exact wiki page title to fetch.")
    fetch_parser.add_argument(
        "--content-format",
        choices=["html", "wikitext"],
        default="html",
        help="Which representation of the page to output (default: html).",
    )
    fetch_parser.set_defaults(func=command_fetch)

    save_parser = subparsers.add_parser(
        "save", help="Download wiki wikitext and store it in a Markdown file."
    )
    save_parser.add_argument("title", help="Exact wiki page title to fetch.")
    save_parser.add_argument(
        "--output-dir",
        help="Directory for the generated wikitext file (default: assets/nesdev).",
        default=str(default_output_dir()),
    )
    save_parser.add_argument(
        "--output-path",
        help="Explicit file path for the wikitext file (overrides --output-dir).",
    )
    save_parser.add_argument(
        "--slug",
        help="Override the filename slug (defaults to the wiki title).",
    )
    save_parser.add_argument(
        "--author",
        default="NESDev contributors",
        help='Author field for the metadata template (default: "NESDev contributors").',
    )
    save_parser.add_argument(
        "--date",
        help="Override the metadata date (default: current UTC date).",
    )
    save_parser.set_defaults(func=command_save)

    parser.set_defaults(func=command_list)
    return parser


def clamp_batch_size(batch: int) -> int:
    batch = max(1, batch)
    return min(batch, MAX_BATCH_SIZE)


def main(argv: List[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if not args.command:
        args.command = "list"
        # Populate defaults that would typically come from the list subparser.
        args.prefix = getattr(args, "prefix", None)
        args.limit = getattr(args, "limit", 50)
        args.batch_size = getattr(args, "batch_size", MAX_BATCH_SIZE)
        args.output_format = getattr(args, "output_format", "text")
        args.func = command_list

    if hasattr(args, "batch_size"):
        args.batch_size = clamp_batch_size(args.batch_size)

    try:
        args.func(args)
    except (requests.RequestException, WikiError) as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        return 130

    return 0


if __name__ == "__main__":
    sys.exit(main())
