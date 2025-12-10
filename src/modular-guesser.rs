pub fn main() {
    let ingots = [
        412924554,   // astroloy
        1058547521,  // constantan
        -404336834,  //
        502280180,   //
        226410516,   //
        1579842814,  //
        -787796599,  //
        -297990285,  //
        -1301215609, //
        2134647745,  //
        -1406385572, //
        -290196476,  //
        -929742000,  //
        -82508479,   //
        -654790771,  //
        -1897868623, //
        156348098,
    ];

    guess_modulus(&ingots);

    let stuff = [
        -1826855889, // wall
        -524546923,  // iron wall
        38555961,    // steel sheet
        662053345,   // plastic sheet
        1588896491,  // glass sheet
        -466050668,  // cable
        2060134443,  // heavy cable
        -1448105779, // steel frame
        -1619793705, // pipe
        452636699,   // insulated pipe
        2067655311,  // insulated liquid pipe
        -744098481,  // integrated cirtuit
        1512322581,  //ItemKitLogicCircuit
    ];
    guess_modulus2(&stuff);
}

fn guess_modulus(hashes: &[i32]) {
    for modulus in hashes.len().. {
        let slots = count_modulus_hash2(hashes, modulus, |x| x.rem_euclid(modulus as i32) as usize);
        if !has_collision(&slots) {
            println!("{modulus} for {}", hashes.len());
            return;
        } else {
            print!(".");
        }
    }
}

fn guess_modulus2(hashes: &[i32]) {
    for modulus1 in hashes.len().. {
        for modulus2 in hashes.len()..=modulus1 {
            let slots = count_modulus_hash2(hashes, modulus2, |x| {
                let stage1 = x.rem_euclid(modulus1 as i32) as usize;
                stage1.rem_euclid(modulus2)
            });
            if !has_collision(&slots) {
                println!("{modulus1}.{modulus2} for {}", hashes.len());
                return;
            } else {
                print!(".");
            }
        }
    }
}

fn count_modulus_hash2(raw: &[i32], slot_count: usize, modulus: impl Fn(i32) -> usize) -> Vec<u16> {
    let mut rval = vec![0; slot_count];
    for &x in raw {
        let slot = modulus(x);
        rval[slot] += 1;
    }
    rval
}

fn has_collision(slots: &[u16]) -> bool {
    slots.iter().any(|&x| x > 1)
}
