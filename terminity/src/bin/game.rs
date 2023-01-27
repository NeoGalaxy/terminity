use std::io::stdout;
use terminity::games;

use structopt::StructOpt;
#[derive(StructOpt)]
struct MasterOpt {
    #[structopt(required = true)]
    game: String
}
fn main() {
    let opt: MasterOpt = MasterOpt::from_args();
    games::get(&opt.game).expect(&("Unable to find game named ".to_owned() + &opt.game))
        .run(&mut stdout()).unwrap();
}
