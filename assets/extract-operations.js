// http://www.oxyron.de/html/opcodes02.html

// Get the list of ops
tables = document.querySelectorAll("table")
var [,logicalTable, moveTable, jumpTable, illegalTable] = tables

var text = ''
function extractOpCodes(table) {
 	trs = [...table.querySelectorAll('tr')].slice(1);
  for(const tr of trs) {
   	const tds = [...tr.querySelectorAll('td')]
    	.map(td => td.innerText.trim());
    const opcode = tds[0].toLowerCase();
    const documentation = tds[13]
    const [n, v, b, d, i, z, c] = tds.slice(14)
    const flags = []
    if (n) flags.push('N')
    if (v) flags.push('V')
    if (b) flags.push('B')
    if (d) flags.push('D')
    if (i) flags.push('I')
    if (z) flags.push('Z')
    if (c) flags.push('C')

    text += `
// Function: ${documentation}
// Flags: ${flags.join(' ')}
fn ${opcode}(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
    // TODO
    let address = cpu.get_operand_address(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}
`
  }
}

extractOpCodes(logicalTable);
extractOpCodes(moveTable);
extractOpCodes(jumpTable);
extractOpCodes(illegalTable);

copy(text)
