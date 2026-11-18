fn main() {
    greet_world();
}

fn greet_world() {
    let southern_germany = "Grüß Gott!";
    let chinese = "元哥 你好";
    let english = "Liang Hello World!";
    let regions = [southern_germany, chinese, english];

    for region in regions {
        println!("{}", region);
    }
}
