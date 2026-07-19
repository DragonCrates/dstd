#![no_std]
#![no_main]
use dstd::prelude::*;
use dstd::io;

fn input(msg: &str) -> String {
    let mut buf = String::new();
    println!("{msg}");
    io::stdin().read_line(&mut buf).expect("stdin read failed");
    buf
}

fn n(msg: &str) -> f64 {
    input(msg).trim().parse().expect("Not a number!")
}

dstd::main!(main);
fn main() {
    println!("Welcome to CALCULATOR-9000!");
    loop {
        let first = n("Enter first operand: ");

        let sign = input("Enter sign. [+-*/ or exit] ");
        match sign.trim() {
            "+"|"-"|"*"|"/" => {},
            "exit" => {
                println!("Nice seeing you!");
                break
            }
            _ => {
                println!("Unknown command!");
                continue
            }
        }

        let second = n("Enter second operand: ");

        match sign.trim() {
            "+" => println!("{} + {} = {}", first, second, first + second),
            "-" => println!("{} - {} = {}", first, second, first - second),
            "*" => println!("{} * {} = {}", first, second, first * second),
            "/" => println!("{} / {} = {}", first, second, first / second),
            _ => {},
        }
    }
}
