#import "monokai_pro.typ": *

#set page(
  height: auto,
  width: auto,
  fill: rgb("#19181A"),
  margin: 40pt,
)
#set raw(theme: "Monokai Pro.tmTheme")
#set text(size: 18pt, fill: rgb("#FCFCFA"))

#text(
  fill: gradient.linear(blue, green),
)[= Typst] <title-label>


#box()[
  #set text(fill: gradient.linear(red, blue))
  #let rainbow(content) = {
    set text(fill: gradient.linear(..color.map.rainbow))
    box(content)
  }

  This is a gradient on text, but with a #rainbow[twist]!
]

#box()[
  #set text(fill: gradient.linear(red, orange, yellow, green, blue, purple))
  $
    7.32 beta +
    sum_(i=0)^nabla
    (Q_i (a_i - epsilon)) / 2
  $
]

#box()[
  ```rust
  fn main() {
    println!("Hello, world!")
  }
  ```
]

// Luma
#for x in range(250, step: 50) {
  box(square(fill: luma(x)))
}
// Linear gradient
#stack(
  dir: ltr,
  square(fill: gradient.linear(red, blue, angle: 0deg)),
  square(fill: gradient.linear(red, blue, angle: 90deg)),
  square(fill: gradient.linear(red, blue, angle: 180deg)),
  square(fill: gradient.linear(red, blue, angle: 270deg)),
)
// Radial gradient
#stack(
  dir: ltr,
  spacing: 50pt,
  circle(
    fill: gradient.radial(..color.map.viridis),
  ),
  ellipse(
    width: 50pt,
    height: 30pt,
    fill: gradient.radial(
      ..color.map.viridis,
      focal-center: (10%, 40%),
      focal-radius: 5%,
    ),
  ),
)
// Conic gradient
#stack(
  dir: ltr,
  spacing: 50pt,
  circle(
    fill: gradient.conic(..color.map.viridis),
  ),
  circle(
    fill: gradient.conic(
      ..color.map.viridis,
      center: (20%, 30%),
    ),
  ),
)
// Sharpness
#[
  #set rect(width: 400pt, height: 20pt)
  #let grad = gradient.linear(..color.map.rainbow)
  #rect(fill: grad)
  #rect(fill: grad.sharp(5))
  #rect(fill: grad.sharp(5, smoothness: 20%))
]
// Repeat gradient
#circle(
  radius: 40pt,
  fill: gradient.radial(aqua, white).repeat(4),
)

// Image
#stack(
  dir: ltr,
  spacing: 20pt,
  figure(
    image("images/voxell_logo.png", height: 100pt),
    caption: [Voxell Logo Png],
  ),
  figure(
    image("images/voxell_logo.svg", height: 100pt),
    caption: [Voxell Logo Svg],
  ),
)
