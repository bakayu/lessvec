use minivec::MiniVec;

fn main() {
    // Basic push/pop
    println!(">>> Executing: push(1); push(2)");
    let mut v = MiniVec::new();
    v.push(1);
    v.push(2);
    println!("initial: {:?}", v.as_slice());
    println!();

    // Reserve capacity
    println!(">>> Executing: reserve(8) and then print len/capacity");
    v.reserve(8);
    println!("len = {}, capacity = {}", v.len(), v.capacity());
    println!();

    // Insert / remove
    println!(">>> Executing: insert(1, 5); remove(1)");
    v.insert(1, 5);
    println!("after insert: {:?}", v.as_slice());
    let removed = v.remove(1);
    println!("removed element = {}", removed);
    println!();

    // Mutate via as_mut_slice
    println!(">>> Executing: as_mut_slice()[0] = 42");
    v.as_mut_slice()[0] = 42;
    println!("after mutation: {:?}", v.as_slice());
    println!();

    // Drain elements
    println!(">>> Executing: push(100); drain()");
    v.push(100);
    let drained: Vec<_> = v.drain().collect();
    println!("drained: {:?}", drained);
    println!(
        "after drain: len = {}, capacity = {}",
        v.len(),
        v.capacity()
    );
    println!();

    // Consume via into_iter
    println!(">>> Executing: into_iter()");
    let mut v2 = MiniVec::new();
    v2.push(10);
    v2.push(20);
    let collected: Vec<_> = v2.into_iter().collect();
    println!("into_iter collected: {:?}", collected);
    println!();
}
