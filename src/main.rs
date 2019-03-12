//!
//! The binary.
//!

use std::{
    collections::{BTreeSet, HashMap},
    env, fs,
};

use sxd_document::{parser, Package};
use sxd_xpath::{nodeset::Node, Context, Factory, Value};

pub struct Evaluator<'d> {
    package: Package,
    factory: Factory,
    context: Context<'d>,
}

impl<'d> Evaluator<'d> {
    pub fn new(xml: &str, namespaces: &HashMap<&str, &str>) -> Self {
        let package = parser::parse(&xml).expect("Parsing error");
        let factory = Factory::new();

        let mut context = Context::new();
        for (key, value) in namespaces.iter() {
            context.set_namespace(key, value);
        }

        Self {
            package,
            factory,
            context,
        }
    }

    pub fn root(&'d self) -> Node<'d> {
        self.package.as_document().root().into()
    }

    pub fn evaluate(&self, node: Node<'d>, path: &str) -> Value {
        let xpath = self
            .factory
            .build(path)
            .expect("XPath building error")
            .expect("XPath building error");
        xpath
            .evaluate(&self.context, node)
            .expect("XPath evaluation error")
    }
}

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    let xml_path = args[1].to_owned();
    let xpath = args[2].to_owned();
    let prefix = args[3].to_owned();

    println!("File: {}", xml_path);
    println!("Path: {}", xpath);
    println!("Pref: {}", prefix);

    let mut namespaces = HashMap::with_capacity(5);
    namespaces.insert("pbs", "http://schema.pbs.gov.au/");
    namespaces.insert("xml", "http://www.w3.org/XML/1998/namespace");
    namespaces.insert("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
    namespaces.insert("dbk", "http://docbook.org/ns/docbook");
    namespaces.insert("xlink", "http://www.w3.org/1999/xlink");

    let xml = fs::read_to_string(&xml_path).expect("File reading error");
    let evaluator = Evaluator::new(&xml, &namespaces);
    let mut paths = BTreeSet::new();
    if let Value::Nodeset(nodeset) = evaluator.evaluate(evaluator.root(), &xpath) {
        for node in nodeset.iter() {
            let mut parent = node;
            let mut path = vec![node.element().unwrap().name().local_part()];
            while let Value::Nodeset(p) = evaluator.evaluate(parent, "..") {
                match p.iter().take(1).collect::<Vec<Node>>().get(0) {
                    Some(p) => {
                        parent = *p;
                        if let Some(element) = parent.element() {
                            path.push(element.name().local_part());
                        }
                    }
                    None => break,
                }
            }
            path.reverse();
            let path = path
                .iter()
                .map(|e| prefix.to_owned() + e)
                .collect::<Vec<String>>()
                .join("/");
            paths.insert(path);
        }
    }

    for path in paths {
        println!("\"{}\",", path);
    }
    Ok(())
}
