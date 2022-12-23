mod store;

fn init_fbb<'a>() -> flatbuffers::FlatBufferBuilder<'a> {
    let fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
    fbb
}

fn main() {
  let fbb = init_fbb();
}
