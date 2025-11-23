// Extract opcodes
// http://www.oxyron.de/html/opcodes02.html

function nullOrString(a) {
  if (a === null) {
    return null;
  }
  return a[0]
}

function colorToStability(color) {
  if (color === "#0000FF") {
    return "somewhat";
  }
  if (color === "#FF0000") {
   	return "unstable"
  }
  if (!color) {
    return "________";
  }
  throw new Error("Unhandled color")
}


function processTD(td) {
  const text = td.innerText.trim();
	const operation = text.match(/[A-Z]{3}/)[0]
  const cycles = nullOrString(text.match(/[0-9]/));
  let mode = nullOrString(text.match(/[a-z]+/));
  let extraCycles = !!text.match(/\*/);
  const stability = colorToStability(td.querySelector("font").getAttribute("color"))
  const illegal = !!td.getAttribute("bgcolor");

  return {
		operation,
    cycles,
    mode,
    extraCycles,
    stability,
    illegal,
  };
}

table = $('table')
rows = [...table.querySelectorAll('tr')]
  .slice(1)
  .map(row => [...row.querySelectorAll('td')].slice(1))

results = rows.map(row => row.map(td => processTD(td)))

console.log('rows', rows)
console.log('results', results)
console.table(results.flat())

text = ''
i = 0;
for(const row of results) {
  for(const entry of row) {
    let index = i.toString(16)
    if (index.length === 1) {
      index = '0' + index;
    }
    let mode = entry.mode || '___'
    if (mode.length === 2) {
      mode = mode + ' ';
    }
    text += [
      '0x' + index,
      entry.operation || "none",
      entry.cycles || '_',
      mode,
      entry.extraCycles || "____",
      entry.stability || "____",
      entry.illegal || "____",
    ].join(' ') + '\n'
    i++
  }
}
copy(text)
