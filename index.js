const cp = require('node:child_process');

const SyntaxTokenModifier = {
	Constant: 'readonly',
};
const SyntaxTokenType = {
	Class: 'class',
	Function: 'function',
	Variable: 'variable',
	Parameter: 'parameter',
	Module: 'module',
};

try {
	JSON.parse(
		cp.execSync('A:\\GitHub\\agalang-core\\target\\debug\\agalang-core.exe tokens "A:\\GitHub\\agalang-core\\a.aga"', {
			encoding: 'utf8',
		})
	)
		.filter(data => {
			data.definition.line === data.location.start.line && data.definition.column === data.location.start.column;
		});
} catch (e) {
	const error_lines = e.stderr.split('\n');
	if (error_lines[0].startsWith('\x1B[1m\x1B[91merror\x1B[39m:\x1B[0m')) {
		const error = error_lines[0].replace('\x1B[1m\x1B[91merror\x1B[39m:\x1B[0m', '').trim();
		const [column, line] = error_lines[1].split(':').reverse();
		console.log(error, Number(line), Number(column));
	}
}
