use rand::Rng;
use std::io;

fn main() {
       let secret_number = rand::thread_rng().gen_range(1..=100);
       loop{
        let mut guess = String::new();
        println!("Please input your guess.");
        io::stdin().read_line(&mut guess).expect("Filed To get input");

        let guess: u32 =  match guess.trim().parse(){
            Ok(num) => num,
            Err(_) => {
                println!("Please input a valid number");
                continue;
            }
        };

        if guess < secret_number{
            println!("Number is To low");
        }else if guess > secret_number {
            println!("number is To high")
        }else{
            println!("Found it you won");
            break;
        }
       }
    // fn add(x: i32, y: i32) {
    //    println!("Sum ;- {}" ,  x + y)
    // }
    // add(5, 7);

    // fn sqaure(x: i32) -> i64 {
    //     (x * x) as i64
    // }
    // println!("Sqaue :- {}" , sqaure(6));

    // struct FileEntry {
    //     path: String,
    //     size: u64,
    //     indexeed: bool,
    // }

    // let file1 = FileEntry{
    //     path: String::from("/hello/yellow"),
    //     size: 92,
    //     indexeed: true,
    // };

    // println!(" {} size - {} - indexed :-{}" ,file1.path , file1.size , file1.indexeed);
   
}
