fn main() {
    println!(
        "digraph regexp {{ 
  fontname=\"Helvetica,Arial,sans-serif\"
  node [fontname=\"Helvetica,Arial,sans-serif\"]
  edge [fontname=\"Helvetica,Arial,sans-serif\",color=blue]"
    );
    for (i, label) in [(0, "1"), (1, "2")] {
        println!("  n{i} [label=\"{label}\"];");
    }
    // n0 [label="regexp"];
    // n1 [label="bytes"];
    // n2 [label="io"];
    // n3 [label="なああ"];
    // n5 [label="strconv"];
    // n6 [label="strings"];
    // n7 [label="sync"];
    // n8 [label="unicode"];
    // n13 [label="math"];
    // n15 [label="unsafe"];
    println!("}}");
}
