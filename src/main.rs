mod emulation;
pub mod error;
pub mod fourier;
pub mod structs;
mod test;
mod util;

fn main()
{
	if let Err(e) = test::test1() {
		eprintln!("{}", e);
	}
}
