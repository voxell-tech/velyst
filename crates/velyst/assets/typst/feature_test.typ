#import "styles/monokai_pro.typ": *

#let section(label: "", body) = block(
  width: 100%,
  fill: base2,
  stroke: 0.05em + base4,
  inset: 0.7em,
  radius: 0.35em,
  stack(
    text(size: 0.7em, fill: base7)[#label],
    v(0.45em),
    body,
  ),
)

#let feature_test() = {
  set text(size: 12pt, fill: base8)

  grid(
    columns: (20em, 20em, 20em),
    gutter: 0.9em,

    section(label: "Text")[
      Regular text. *Bold text.* _Italic text._ *_Bold italic._*
      #linebreak()
      #text(fill: red)[Colored] #text(fill: blue)[text] #text(
        fill: green,
      )[words.]
      #linebreak()
      #stack(
        dir: ltr,
        spacing: 1.1em,
        text(fill: gradient.linear(red, blue, green))[= Linear],
        text(fill: gradient.radial(yellow, base0))[= Radial],
        text(fill: gradient.conic(
          red,
          orange,
          yellow,
          green,
          blue,
          purple,
          red,
        ))[= Conic],
      )
    ],

    section(label: "Shapes / rect, circle, polygon")[
      #stack(
        dir: ltr,
        spacing: 0.7em,
        rect(width: 5em, height: 5em, fill: blue, radius: 0.35em),
        circle(radius: 2.5em, fill: red),
        polygon(
          fill: green,
          (2.5em, 0em),
          (5em, 4.2em),
          (0em, 4.2em),
        ),
      )
    ],

    section(label: "Stroke styles / dash, caps, joins")[
      #stack(
        spacing: 0.5em,
        line(length: 100%, stroke: (
          paint: base8,
          thickness: 0.15em,
          dash: "dashed",
        )),
        line(length: 100%, stroke: (
          paint: red,
          thickness: 0.25em,
          dash: "dotted",
        )),
        line(length: 100%, stroke: (
          paint: blue,
          thickness: 0.35em,
          cap: "round",
        )),
        rect(
          width: 100%,
          height: 1.5em,
          stroke: (paint: green, thickness: 0.25em, join: "round"),
        ),
      )
    ],

    section(label: "Shape gradients")[
      #grid(
        columns: (1fr, 1fr, 1fr),
        gutter: 0.5em,
        rect(width: 100%, height: 5em, fill: gradient.linear(
          red,
          blue,
          green,
          angle: 45deg,
        )),
        rect(width: 100%, height: 5em, fill: gradient.radial(
          green,
          base0,
          focal-center: (40%, 40%),
        )),
        rect(width: 100%, height: 5em, fill: gradient.conic(
          red,
          orange,
          yellow,
          green,
          blue,
          purple,
          red,
        )),
      )
    ],

    section(label: "Image")[
      #let annotated(label: "", body) = stack(
        body,
        v(0.35em),
        text(size: 0.65em, fill: base5)[#label],
      )
      #stack(
        dir: ltr,
        spacing: 1.4em,
        annotated(label: "raster", image(
          "../images/voxell_logo.png",
          height: 5em,
          fit: "contain",
        )),
        annotated(label: "svg", image(
          "../images/voxell_logo.svg",
          height: 5em,
          fit: "contain",
        )),
      )
    ],

    section(label: "Clipping")[
      #box(
        width: 10em,
        height: 5em,
        clip: true,
        radius: 0.7em,
        fill: base0,
      )[
        #circle(radius: 4.5em, fill: purple)
      ]
    ],

    section(label: "Rotation")[
      #stack(
        dir: ltr,
        spacing: 1.8em,
        rotate(0deg, rect(width: 3.5em, height: 3.5em, fill: blue)),
        rotate(15deg, rect(width: 3.5em, height: 3.5em, fill: red)),
        rotate(30deg, rect(width: 3.5em, height: 3.5em, fill: green)),
        rotate(45deg, rect(width: 3.5em, height: 3.5em, fill: purple)),
      )
    ],

    section(label: "Color effects")[
      #let swatch(color) = rect(
        width: 1.2em,
        height: 3em,
        fill: color,
        radius: 0.15em,
      )
      #let effect(label: "", swatches) = stack(
        text(size: 0.65em, fill: base5)[#label],
        v(0.35em),
        stack(dir: ltr, spacing: 0.25em, ..swatches),
      )

      #grid(
        columns: (1fr, 1fr, 1fr),
        gutter: 0.7em,
        effect(
          label: "transparentize",
          range(4).map(i => swatch(blue.transparentize(i * 30%))),
        ),
        effect(
          label: "lighten",
          range(4).map(i => swatch(red.lighten(i * 25%))),
        ),
        effect(
          label: "darken",
          range(4).map(i => swatch(green.darken(i * 25%))),
        ),

        effect(
          label: "saturate",
          range(4).map(i => swatch(orange.saturate(i * 25%))),
        ),
        effect(
          label: "desaturate",
          range(4).map(i => swatch(purple.desaturate(i * 25%))),
        ),
        effect(
          label: "hue rotate",
          range(4).map(i => swatch(red.rotate(i * 72deg))),
        ),
      )
    ],

    section(label: "Math equations")[
      $ E = m c^2 $
      #h(1.4em)
      $ integral_0^1 x^2 dif x = 1/3 $
      #h(1.4em)
      $ sum_(n=1)^oo 1/n^2 = pi^2/6 $
    ],

    section(label: "Table")[
      #let done = text(fill: green)[Done]
      #table(
        columns: (auto, 1fr, 1fr),
        fill: (col, row) => if row == 0 { base0 } else if calc.odd(row) {
          base2
        } else { base3 },
        stroke: 0.05em + base4,
        table.header(
          text(fill: yellow)[Feature],
          text(fill: yellow)[Status],
          text(fill: yellow)[Notes],
        ),
        [Text], done, [solid, gradient],
        [Shapes], done, [rect, circle, poly],
        [Images], done, [raster, SVG],
        [Gradients], done, [linear, radial, conic],
        [Clipping], done, [arbitrary paths],
        [Math], done, [via math fonts],
      )
    ],
  )
}
