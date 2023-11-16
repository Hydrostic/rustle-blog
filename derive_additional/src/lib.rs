pub trait MessagePrintable: Sized{
    fn print(&self) -> String;
}