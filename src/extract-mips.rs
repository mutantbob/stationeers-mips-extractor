use minidom::Element;
use rxml::{EventRead, Lexer, PullDriver, RawEvent, RawParser, RawQName};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

/// parse a Stationeers world file and extract all the MISP routines from the IC10 objects
/// c:\\Users\\*\\Documents\\My Games\\Stationeers\\saves\\*\\world.xml
fn main() -> Result<(), Box<dyn Error>> {
    let (fname, out_dir) = {
        let mut args = std::env::args();
        let _arg0 = args.next();
        (
            args.next().unwrap(),
            args.next().unwrap_or_else(|| "/tmp/mips".to_owned()),
        )
    };

    println!("file {}", fname);

    let mut file = File::open(fname)?;
    if false {
        let mut buf = [0; 16];
        let x = file.read(&mut buf);
        println!("{:#?}", x);
        println!("{:#?}", buf);
    }
    let mut reader = BufReader::new(file);

    if false {
        let mut buf = [0; 16];
        let x = reader.read(&mut buf);
        println!("{:#?}", x);
        println!("{:#?}", buf);
    }

    if true {
        let mut driver = PullDriver::wrap(reader, Lexer::new(), RawParser::new());
        extract_mips(&mut driver, PathBuf::from(out_dir))?;
    } else {
        exp1(reader)?;
    }

    Ok(())
}

#[derive(Default)]
struct ThingSaveData {
    reference_id: Option<String>,
    source: Option<String>,
    qname_stack: Vec<RawQName>,
}

impl ThingSaveData {
    pub fn add_source(&mut self, more_source: &str) {
        if let Some(source) = &mut self.source {
            source.push_str(more_source);
        } else {
            self.source = Some(more_source.to_owned());
        }
    }

    pub fn finish(&self, mips_sink: &mut MipsSink) {
        if let Some(source) = &self.source {
            if let Some(reference_id) = &self.reference_id {
                if let Err(e) = mips_sink.accept(reference_id, source) {
                    println!("{:?}", e);
                }
            }
        }
    }
}

//

struct MipsSink {
    out_dir: PathBuf,
}

impl MipsSink {
    pub(crate) fn accept(&self, reference_id: &str, source: &str) -> Result<(), std::io::Error> {
        let out_path = self.out_dir.join(format!("{}.mips", reference_id));
        let mut file = File::create(&out_path)?;
        write!(&mut file, "{}", source)?;
        println!("wrote {:?}", out_path.to_str());
        Ok(())
    }
}

//

// find
fn extract_mips(
    driver: &mut PullDriver<BufReader<File>, RawParser>,
    out_dir: PathBuf,
) -> Result<(), rxml::Error> {
    let mut mips_sink = MipsSink { out_dir };
    let mut ctx: Option<ThingSaveData> = None;
    println!("go");
    while let Some(event) = driver.read()? {
        match event {
            RawEvent::XmlDeclaration(_, _) => {
                println!("{:#?}", event);
            }
            RawEvent::ElementHeadOpen(_metrics, name) => {
                let name2 = name.1.clone().into_name();
                if name2.as_str() == "ThingSaveData" {
                    ctx = Some(Default::default())
                } else if let Some(ctx) = &mut ctx {
                    ctx.qname_stack.push(name);
                }
            }
            RawEvent::Attribute(_, _, _) => {}
            RawEvent::ElementHeadClose(_) => {}
            RawEvent::ElementFoot(_metrics) => {
                if let Some(ctx2) = &mut ctx {
                    if ctx2.qname_stack.pop().is_none() {
                        ctx2.finish(&mut mips_sink);
                        ctx = None;
                    }
                }
            }
            RawEvent::Text(_metrics, c_data) => {
                if let Some(ctx) = &mut ctx {
                    if let Some((_, qname)) = ctx.qname_stack.last() {
                        let qname_str = qname.as_str();
                        match qname_str {
                            "SourceCode" => {
                                ctx.add_source(c_data.as_str());
                            }
                            "ReferenceId" => {
                                ctx.reference_id = Some(c_data.as_str().to_owned());
                            }
                            "DamageState"
                            | "DynamicThing"
                            | "IsCustomName"
                            | "WorldRotation"
                            | "w"
                            | "x"
                            | "y"
                            | "z"
                            | "eulerAngles"
                            | "Brute"
                            | "Radiation"
                            | "WorldPosition"
                            | "HasSpawnedWreckage"
                            | "MothershipReferenceId"
                            | "CurrentBuildState"
                            | "Decay"
                            | "Stun"
                            | "Toxic"
                            | "Starvation"
                            | "Hydration"
                            | "Oxygen"
                            | "Burn"
                            | "Indestructable"
                            | "OwnerSteamId"
                            | "CableNetworkId"
                            | "CustomColorIndex"
                            | "PrefabName"
                            | "AngularVelocity"
                            | "Velocity"
                            | "DragOffset"
                            | "State"
                            | "StateName"
                            | "NextNeighborId"
                            | "ChuteNetworkId"
                            | "IsBurst"
                            | "PipeNetworkId"
                            | "HealthCurrent"
                            | "States"
                            | "MasterMotherboard" => {} // ignore

                            _ => {
                                //println!("qname {}", qname_str);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn scan_mips(driver: &mut PullDriver<BufReader<File>, RawParser>) -> Result<(), rxml::Error> {
    let mut path = vec![];
    println!("go");
    while let Some(event) = driver.read()? {
        match event {
            RawEvent::XmlDeclaration(_, _) => {
                println!("{:#?}", event);
            }
            RawEvent::ElementHeadOpen(_metrics, name) => {
                let name2 = name.1.clone().into_name();
                path.push(name);
                if name2.as_str() == "SourceCode" {
                    println!("{}\t{:?}", name2, path);
                }
            }
            RawEvent::Attribute(_, _, _) => {}
            RawEvent::ElementHeadClose(_) => {}
            RawEvent::ElementFoot(_metrics) => {
                let x = path.pop();
                if x.is_none() {
                    println!("malfunction, popped too hard");
                }
            }
            RawEvent::Text(_, _) => {}
        }
    }

    Ok(())
}

/// fails with MissingNamespace
fn exp1(reader: BufReader<File>) -> Result<(), minidom::Error> {
    let root: Element = Element::from_reader(reader)?;

    println!("{:#?}", root);
    Ok(())
}
