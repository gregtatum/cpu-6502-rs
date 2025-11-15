#!/usr/bin/env python3
"""
Utility CLI for pulling down NESDev wiki content as local references.

The streamlined workflow is:
  * search for relevant pages
  * save the wikitext (with local metadata) to assets/nesdev
  * optionally use "grab" to search and save in one step

Results are cached on disk so repeated queries are fast/offline-friendly.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple

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

    response = requests.get(
        API_URL, params=params, headers=CLOUDFLARE_HEADERS, timeout=10
    )
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


def search_pages(term: str, limit: int, refresh: bool) -> List[Dict]:
    params = {
        "action": "query",
        "format": "json",
        "list": "search",
        "srsearch": term,
        "srlimit": str(limit),
    }
    data = cached_request(params, refresh=refresh)
    return data.get("query", {}).get("search", [])


def print_search_results(results: List[Dict]) -> None:
    if not results:
        print("No results found.")
        return

    for idx, result in enumerate(results, start=1):
        title = result["title"]
        snippet = (
            result.get("snippet", "")
            .replace('<span class="searchmatch">', "")
            .replace("</span>", "")
        )
        slug = title_to_slug(title)
        url = f"https://www.nesdev.org/wiki/{slug}"
        print(f"{idx}. {title}")
        if snippet:
            print(f"   {snippet}")
        print(f"   URL: {url}")
        print(f'   Save: uv run nesdev.py save "{title}"')


def command_search(args: argparse.Namespace) -> None:
    results = search_pages(args.term, args.limit, args.refresh)
    print_search_results(results)


def title_to_slug(title: str) -> str:
    """Convert a wiki title into a filesystem-friendly slug."""
    return title.replace(" ", "_")


def default_output_dir() -> Path:
    override = os.environ.get("NESDEV_REF_DIR")
    if override:
        return Path(override).expanduser()
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


def save_wikitext_document(
    title: str, refresh: bool, output_dir: Path | None = None
) -> Path:
    parsed, wikitext = fetch_wikitext(title, refresh=refresh)
    slug = title_to_slug(parsed.get("title", title))

    final_dir = output_dir or default_output_dir()
    final_dir.mkdir(parents=True, exist_ok=True)
    output_path = final_dir / f"{slug}.wikitext"

    metadata = {
        "title": parsed.get("title"),
        "author": "NESDev contributors",
        "date": datetime.now(timezone.utc).date().isoformat(),
        "source": f"https://www.nesdev.org/wiki/{title_to_slug(parsed.get('title', title))}",
        "pageid": parsed.get("pageid"),
        "revid": parsed.get("revid"),
    }

    front_matter = format_metadata_wikitext(metadata)
    content = wikitext.rstrip("\n")
    document = f"{front_matter}\n\n{content}\n"

    with output_path.open("w", encoding="utf-8") as outfile:
        outfile.write(document)

    return output_path


def command_save(args: argparse.Namespace) -> None:
    output_path = save_wikitext_document(args.title, refresh=args.refresh)
    print(f"Wrote {output_path}")


def command_grab(args: argparse.Namespace) -> None:
    results = search_pages(args.term, args.limit, args.refresh)
    print_search_results(results)
    if not results:
        return

    if args.pick is None:
        print('Re-run with "--pick N" to download a result automatically.')
        return

    index = args.pick - 1
    if index < 0 or index >= len(results):
        raise WikiError(
            f"Pick value {args.pick} is out of range for {len(results)} results."
        )

    title = results[index]["title"]
    output_path = save_wikitext_document(title, refresh=args.refresh)
    print(f"Saved result #{args.pick} ({title}) to {output_path}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="NESDev MediaWiki helper utility.")
    parser.add_argument(
        "--refresh",
        action="store_true",
        help="Ignore the local cache and fetch fresh data from the wiki.",
    )
    subparsers = parser.add_subparsers(dest="command")

    search_parser = subparsers.add_parser("search", help="Search for pages.")
    search_parser.add_argument("term", help="Search query.")
    search_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="Maximum number of results to return (default: 20).",
    )
    search_parser.set_defaults(func=command_search)

    save_parser = subparsers.add_parser(
        "save", help="Download wiki wikitext and store it in assets/nesdev."
    )
    save_parser.add_argument("title", help="Exact wiki page title to fetch.")
    save_parser.set_defaults(func=command_save)

    grab_parser = subparsers.add_parser(
        "grab",
        help="Search and optionally save a result in one command.",
    )
    grab_parser.add_argument("term", help="Search query.")
    grab_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="Maximum number of search results to scan (default: 20).",
    )
    grab_parser.add_argument(
        "--pick",
        type=int,
        help="Automatically save the Nth result (1-based).",
    )
    grab_parser.set_defaults(func=command_grab)

    return parser


def main(argv: List[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if not hasattr(args, "func"):
        parser.print_help()
        return 1

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
