use std::process::Command;
use wasmtime::*;

fn main() {
    match std::env::args().nth(1) {
        Some(i) => {
            let i = i.parse::<usize>().unwrap();
            let mut s = String::new();
            s.push_str("(module\n");
            for i in 0..400 {
                s.push_str(&format!("(func call {})\n", i));
            }
            s.push_str(&format!("(func $f0 unreachable)\n"));
            for j in 1..100 {
                s.push_str(&format!("(func $f{} call $f{})\n", j, j - 1));
            }
            s.push_str(&format!("(func (export \"\") call $f99)\n"));
            s.push_str(")");

            drop(std::fs::remove_dir_all("./moduletmp"));
            std::fs::create_dir("./moduletmp").unwrap();

            let engine = Engine::default();
            let m = Module::new(&engine, &s).unwrap();
            std::fs::write("./moduletmp/image", m.serialize().unwrap()).unwrap();

            let modules = (0..1 << i)
                .map(|j: usize| {
                    let path = format!("./moduletmp/image{}", j);
                    std::fs::copy("./moduletmp/image", &path).unwrap();
                    let x = std::time::Instant::now();
                    let m = unsafe { Module::deserialize_file(&engine, &path).unwrap() };
                    if j == (1 << i) - 3 || j == 3 {
                        println!("{:?}", x.elapsed());
                    }
                    m
                })
                .collect::<Vec<_>>();
            let mut store = Store::new(&engine, ());
            let i = Instance::new(&mut store, &modules[0], &[]).unwrap();
            let f = i.get_typed_func::<(), (), _>(&mut store, "").unwrap();

            let x = std::time::Instant::now();
            assert!(f.call(&mut store, ()).is_err());
            let a = x.elapsed();

            let x = std::time::Instant::now();
            assert!(f.call(&mut store, ()).is_err());
            let b = x.elapsed();

            println!("{:10} {:.02?} {:.02?}", modules.len(), a, b);
        }
        None => {
            for i in 10..15 {
                let x = Command::new(std::env::current_exe().unwrap())
                    .arg(i.to_string())
                    .status()
                    .unwrap();
                assert!(x.success());
            }
        }
    }
}
