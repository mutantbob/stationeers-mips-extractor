use minidom::Element;
use rxml::{Event, GenericReader, QName};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::PathBuf;
use zip::ZipArchive;

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

    let mut file = File::open(&fname).expect(&format!("cannot open {fname} for read"));
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

    match SaveFormat::guess(&mut reader) {
        SaveFormat::XML => {
            let mut r2 = rxml::Reader::new(reader);
            extract_mips(&mut r2, PathBuf::from(out_dir))?;
        }
        SaveFormat::Zip => {
            let mut archive =
                ZipArchive::new(reader).expect(&format!("failed to parse {fname} as zip"));
            let zip_file = archive
                .by_name("world.xml")
                .expect("failed to extract world.xml from zip");
            let zip_file = BufReader::new(zip_file);
            let mut r2 = rxml::Reader::new(zip_file);
            extract_mips(&mut r2, PathBuf::from(out_dir))?;
        }
    }

    Ok(())
}

#[derive(Default)]
struct ThingSaveData {
    reference_id: Option<String>,
    source: Option<String>,
    qname_stack: Vec<QName>,
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
                    println!("malfunction in finish {:?}", e);
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
        let mut file = File::create(&out_path).map_err(|e| {
            std::io::Error::other(format!(
                "Unable to open {} for write: {e}",
                out_path.display()
            ))
        })?;
        write!(&mut file, "{}", source)?;
        if !source.ends_with('\n') {
            writeln!(&mut file)?;
        }
        println!("wrote {:?}", out_path.to_str());
        Ok(())
    }
}

//

// find
fn extract_mips<R: std::io::BufRead, P: rxml::Parse<Output = rxml::Event>>(
    driver: &mut GenericReader<R, P>,
    out_dir: PathBuf,
) -> Result<(), rxml::Error> {
    let mut mips_sink = MipsSink { out_dir };
    let mut ctx: Option<ThingSaveData> = None;
    println!("go");

    while let Some(event) = driver.read()? {
        match event {
            Event::XmlDeclaration(_, _) => {
                println!("{:#?}", event);
            }
            Event::StartElement(_metrics, name, _attrs) => {
                let name2 = name.1.clone().into_name();
                let name2_str = name2.as_str();
                // println!("<{name2_str}",);

                if name2_str == "ThingSaveData" {
                    ctx = Some(Default::default())
                } else if let Some(ctx) = &mut ctx {
                    ctx.qname_stack.push(name);
                }
            }
            Event::EndElement(_) => {
                // println!("</>");
                if let Some(ctx2) = &mut ctx {
                    if ctx2.qname_stack.pop().is_none() {
                        ctx2.finish(&mut mips_sink);
                        ctx = None;
                    }
                }
            }
            Event::Text(_metrics, c_data) => {
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
        };
    }

    Ok(())
}

/// fails with MissingNamespace
fn exp1(reader: BufReader<File>) -> Result<(), minidom::Error> {
    let root: Element = Element::from_reader(reader)?;

    println!("{:#?}", root);
    Ok(())
}

enum SaveFormat {
    XML,
    Zip,
}

impl SaveFormat {
    pub fn guess<R: BufRead + Seek>(mut r: R) -> Self {
        let mut magic = [0; 2];
        if let Err(..) = r.read_exact(&mut magic) {
            return Self::XML;
        };
        r.seek(std::io::SeekFrom::Start(0))
            .expect("failed to seek to start");

        if &magic == b"PK" {
            Self::Zip
        } else {
            Self::XML
        }
    }
}

#[cfg(test)]
mod test;
