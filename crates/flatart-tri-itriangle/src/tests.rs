use crate::ITriangulator;
use flatart_ports::testkit::triangulator;

#[test]
fn conforms() {
    triangulator::run(&ITriangulator);
}
