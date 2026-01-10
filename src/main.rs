mod parser;
mod field;
mod formatter;
mod dictionary;

fn main() -> std::io::Result<()> {
	parser::process(&mut std::io::stdin().lock(), &mut std::io::stdout())
}
