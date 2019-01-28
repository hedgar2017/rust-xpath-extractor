//!
//! The binary.
//!

use std::{collections::BTreeSet, fs};

use sxd_document::{parser, Package};
use sxd_xpath::{nodeset::Node, Context, Factory, Value};

pub struct Evaluator<'d> {
    package: Package,
    factory: Factory,
    context: Context<'d>,
}

impl<'d> Evaluator<'d> {
    pub fn new(xml: &str) -> Self {
        let package = parser::parse(&xml).expect("Parsing error");
        let factory = Factory::new();

        let mut context = Context::new();
        context.set_namespace("pbs", "http://schema.pbs.gov.au/");
        context.set_namespace("xml", "http://www.w3.org/XML/1998/namespace");
        context.set_namespace("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
        context.set_namespace("dbk", "http://docbook.org/ns/docbook");
        context.set_namespace("xlink", "http://www.w3.org/1999/xlink");

        Self {
            package,
            factory,
            context,
        }
    }

    pub fn root(&'d self) -> Node<'d> {
        self.package.as_document().root().into()
    }

    pub fn evaluate(&self, node: Node<'d>, path: &'static str) -> Value {
        let xpath = self.factory.build(path).expect("XPath building error").expect("XPath building error");
        xpath.evaluate(&self.context, node).expect("XPath evaluation error")
    }
}

fn main() -> Result<(), ()> {
    let xml = fs::read_to_string("data.xml").expect("File reading error");
    let evaluator = Evaluator::new(&xml);
    let mut paths = BTreeSet::new();
    if let Value::Nodeset(nodeset) = evaluator.evaluate(evaluator.root(), "//pbs:fee") {
        for node in nodeset.iter() {
            let mut parent = node;
            let mut path = vec!["fee"];
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
                .map(|e| "pbs:".to_string() + e)
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
