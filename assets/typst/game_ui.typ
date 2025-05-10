#import "styles/monokai_pro.typ": *

#let default_size = 16pt

#let lbl(body, fill: base7, size: default_size) = {
  set text(size: size, fill: fill)
  body
}

#let button(body, hovered, fill: base7, size: default_size) = {
  set text(size: size, fill: if hovered { base0 } else { fill })
  box(
    inset: 0.7em,
    fill: if hovered { fill } else { none },
    radius: 0.4em,
  )[#body]
}

#let perf_metrics(fps, elapsed_time) = {
  set text(size: 14pt, fill: base7)

  box(inset: 1em)[
    #box(fill: base0.transparentize(60%), inset: 15pt, radius: 4pt)[
      #align(left)[
        #box(inset: (bottom: 8pt))[
          #text(
            fill: gradient.linear(
              red,
              orange,
              yellow,
              green,
              blue,
              purple,
            ),
          )[= Performance Metrics]
        ]\
        FPS: #fps\
        Elapsed Time: #elapsed_time\
      ]
    ]
  ]
}
