#[derive(Clone, Debug)]
pub enum NodeType {
    MainNode = -1,
    RoundedSegment = 0,
    Segment = 1,
    Circle = 2,
    Triangle = 3,
    FilledCircle = 4,
    Ellipse = 5,
    Trapezoid = 6,
    Polygon = 7,
}

impl NodeType {
    pub fn from_integer(int: i8) -> Option<Self> {
        match int {
            -1 => Some(NodeType::MainNode),
            0 => Some(NodeType::RoundedSegment),
            1 => Some(NodeType::Segment),
            2 => Some(NodeType::Circle),
            3 => Some(NodeType::Triangle),
            4 => Some(NodeType::FilledCircle),
            5 => Some(NodeType::Ellipse),
            6 => Some(NodeType::Trapezoid),
            7 => Some(NodeType::Polygon),
            _ => None,
        }
    }
    pub fn to_integer(&self) -> i8 {
        self.clone() as i8
    }
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Segment
    }
}