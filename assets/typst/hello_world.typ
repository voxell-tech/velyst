#import "styles/monokai_pro.typ": *

#let PI = 3.142

#let wave_gen(func, frequency, amplitude, time, resolution) = {
  let result = ()
  let inv_resolution = 1.0 / resolution

  for i in range(0, resolution + 1) {
    result.push((
      100% * i * inv_resolution,
      (func((time + PI * float(i) / resolution) * frequency) * amplitude) - 50%,
    ))
  }

  return result
}

#let main(
  width,
  height,
  animate: 0.0,
) = {
  let width = (width * 1pt)
  let height = (height * 1pt)

  let amplitude = 10%
  let frequency = 1
  let resolution = 20
  let animate = animate * 2

  box(
    width: width,
    height: height,
  )[
    #set text(size: 48pt, fill: base7)
    #place(
      center,
      dy: 20%,
    )[= Wave]

    #place(center + bottom)[
      #polygon(
        fill: blue.transparentize(90%),
        stroke: blue,
        (0%, 0%),
        ..wave_gen(calc.sin, frequency, amplitude, animate, resolution),
        (100%, 0%),
      )
    ]

    #place(center + bottom)[
      #polygon(
        fill: red.transparentize(90%),
        stroke: red,
        (0%, 0%),
        ..wave_gen(calc.cos, frequency, amplitude, animate, resolution),
        (100%, 0%),
      )
    ]
  ]
}
