#set page(
  height: auto,
  width: auto,
  fill: rgb("#19181A"),
)
#set raw(theme: "Monokai Pro.tmTheme")
#set text(size: 24pt, fill: rgb("#FCFCFA"))

#text(
  fill: gradient.linear(rgb("#13A8C1"), rgb("#21C0AA")),
)[= Typst] <title-label>

#[Total displaced soil by glacial flow:] <text-label>

$
  7.32 beta +
  sum_(i=0)^nabla
  (Q_i (a_i - epsilon)) / 2
$ <math-label>

#lorem(10) <lorem-label>

```rust
fn main() {
  println!("Hello, world!")
}
``` <raw-label>

#box() <box-label>
#block() <block-label>

#circle(radius: 25pt) <circle-label>
#ellipse(width: 35pt, height: 30pt) <ellipse-label>
#line(
  length: 4cm,
  stroke: 2pt + maroon,
) <line-label>
#path(
  fill: blue,
  stroke: red,
  closed: true,
  (0pt, 50pt),
  (100pt, 50pt),
  ((50pt, 0pt), (40pt, 0pt)),
) <path-label>
#polygon(
  fill: blue,
  stroke: red,
  (20pt, 0pt),
  (60pt, 0pt),
  (80pt, 2cm),
  (0pt, 2cm),
) <polygon-label>
#rect(width: 20pt, height: 30pt) <rect-label>
#square(size: 40pt) <square-label>
