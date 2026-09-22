#import "styles/monokai_pro.typ": *

#let strokes = (2pt, 6pt, 12pt, 20pt)

#let main() = {
  box(width: 100%, height: 100%)[
    #set text(size: 72pt, fill: yellow, weight: "bold")
    #set align(center + horizon)
    #stack(
      dir: ttb,
      spacing: 1em,
      ..strokes.map(pt => text(stroke: pt + red)[WAVE #pt]),
    )
  ]
}
