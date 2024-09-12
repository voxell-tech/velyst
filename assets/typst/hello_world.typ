#import "styles/monokai_pro.typ": *

#set page(
  width: auto,
  height: auto,
  fill: black,
  margin: 0pt,
)

#let PI = 3.142

#let main(
  width,
  height,
  animate: 0.0,
) = {
  let width = (width * 1pt)
  let height = (height * 1pt)

  box(
    width: width,
    height: height,
  )[
    #set text(size: 48pt, fill: base7)
    #place(center, dy: 20%)[= Wave]

    #let wave_height = 10%
    #place(center + bottom)[
      #polygon(
        fill: blue.transparentize(94%),
        stroke: blue,
        // closed: true,
        (0%, 0%),
        (0%, (calc.sin(animate) * wave_height) + -50%),
        (10%, (calc.sin(animate + PI * 0.1) * wave_height) + -50%),
        (20%, (calc.sin(animate + PI * 0.2) * wave_height) + -50%),
        (30%, (calc.sin(animate + PI * 0.3) * wave_height) + -50%),
        (40%, (calc.sin(animate + PI * 0.4) * wave_height) + -50%),
        (50%, (calc.sin(animate + PI * 0.5) * wave_height) + -50%),
        (60%, (calc.sin(animate + PI * 0.6) * wave_height) + -50%),
        (70%, (calc.sin(animate + PI * 0.7) * wave_height) + -50%),
        (80%, (calc.sin(animate + PI * 0.8) * wave_height) + -50%),
        (90%, (calc.sin(animate + PI * 0.9) * wave_height) + -50%),
        (100%, (calc.sin(animate + PI) * wave_height) + -50%),
        (100%, 0%),
      )
    ]

    #place(center + bottom)[
      #polygon(
        fill: red.transparentize(94%),
        stroke: red,
        // closed: true,
        (0%, 0%),
        (0%, (calc.cos(animate) * wave_height) + -50%),
        (10%, (calc.cos(animate + PI * 0.1) * wave_height) + -50%),
        (20%, (calc.cos(animate + PI * 0.2) * wave_height) + -50%),
        (30%, (calc.cos(animate + PI * 0.3) * wave_height) + -50%),
        (40%, (calc.cos(animate + PI * 0.4) * wave_height) + -50%),
        (50%, (calc.cos(animate + PI * 0.5) * wave_height) + -50%),
        (60%, (calc.cos(animate + PI * 0.6) * wave_height) + -50%),
        (70%, (calc.cos(animate + PI * 0.7) * wave_height) + -50%),
        (80%, (calc.cos(animate + PI * 0.8) * wave_height) + -50%),
        (90%, (calc.cos(animate + PI * 0.9) * wave_height) + -50%),
        (100%, (calc.cos(animate + PI) * wave_height) + -50%),
        (100%, 0%),
      )
    ]
  ]
}
