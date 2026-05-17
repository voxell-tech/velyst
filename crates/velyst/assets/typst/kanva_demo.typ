#import "styles/monokai_pro.typ": *

#let layer-counts = (8, 5, 3, 5, 8)
#let layer-colors = (yellow, green, blue, green, purple)
#let node-r = 10pt
#let spacing = 28pt
#let bw = 500pt
#let bh = 280pt
#let layer-xs = (50pt, 150pt, 250pt, 350pt, 450pt)

#let node-positions = {
  let result = ()
  for (i, count) in layer-counts.enumerate() {
    let start-y = (bh - (count - 1) * spacing) / 2
    let layer = ()
    for j in range(count) {
      layer.push((layer-xs.at(i), start-y + j * spacing))
    }
    result.push(layer)
  }
  result
}

#let main() = {
  set text(fill: base7, size: 8pt)

  box(width: bw, height: bh)[
    // Connection lines — animated via kanva.
    #box[
      #for i in range(layer-counts.len() - 1) {
        for a in node-positions.at(i) {
          for b in node-positions.at(i + 1) {
            place(top + left, line(start: a, end: b, stroke: base4 + 0.5pt))
          }
        }
      }
    ] <connections>

    // Nodes drawn on top.
    #for (i, layer) in node-positions.enumerate() {
      for pos in layer {
        place(
          top + left,
          dx: pos.at(0) - node-r,
          dy: pos.at(1) - node-r,
          circle(radius: node-r, fill: base2, stroke: layer-colors.at(i) + 1pt),
        )
      }
    }
  ]
}
