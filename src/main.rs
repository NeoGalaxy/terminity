use crate::games::CLGame;

mod games;

fn main() {
    /*for x in 0..3 {
        println!("===========");
        for y in 0..3 {

            println!("{:?} | {:?}, {:?}, {:?}",
                x + y == 2, (x, y), ((x + 1) % 3, (y + 2) % 3), ((x + 2) % 3, (y + 1) % 3));

            println!("{:?} | {:?}, {:?}, {:?}",
                x + y == 2, (x, y), ((x + 1) % 3, (y + 2) % 3), ((x + 2) % 3, (y + 1) % 3));

            println!("{:?} | {:?}, {:?}, {:?}",
                x + y == 2, (x, y), ((x + 1) % 3, (y + 2) % 3), ((x + 2) % 3, (y + 1) % 3));
        }
    }*/
    games::sttt::SuperTTT::run(console::Term::stdout()).unwrap();
}
