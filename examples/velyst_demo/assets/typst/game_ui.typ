#import "styles/monokai_pro.typ": *

#let default_size = 24pt

#let lbl(body, fill: base7, size: default_size) = {
  set text(size: size, fill: fill)
  body
}

#let button(body, interaction_state, fill: base7, size: default_size) = {
  set text(size: size, fill: fill)
  box(
    inset: 0.7em,
    fill: if interaction_state != 0 { base4 } else { none },
    stroke: if interaction_state == 2 { fill + 0.15em } else { none },
    radius: 0.6em,
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
