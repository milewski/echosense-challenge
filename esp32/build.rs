fn main() {

    if let Err(error) = dotenv_build::output(dotenv_build::Config::default()) {
        panic!(".env file not found. {}", error);
    };

    embuild::espidf::sysenv::output();

}
