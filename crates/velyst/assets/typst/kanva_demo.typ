#import "@preview/cetz:0.5.2": canvas, draw
#import draw: content, line
#import "styles/monokai_pro.typ": *

#let spacing = 1.2
#let layer_spacing = 4

#let layer_defs = (
  (8, yellow, "x", none),
  (5, base6, "h", "1"),
  (3, base6, "z", none),
  (5, base6, "h", "2"),
  (8, purple, "hat(x)", none),
)

#let layers = {
  let result = ()
  for (i, def) in layer_defs.enumerate() {
    let (count, fill, prefix, sup) = def
    result.push((i * layer_spacing, count, fill, prefix, sup))
  }
  result
}

#let neuron(pos, fill: white, text_fill: base0, text: none) = {
  draw.content(
    pos,
    std.text(fill: text_fill, text),
    frame: "circle",
    fill: fill,
    stroke: 2pt + fill.lighten(50%),
    padding: 2pt,
  )
}

#let connect_layers(start_pos, start_count, end_pos, end_count) = {
  let start_y = (start_count - 1) / 2 * spacing
  let end_y = (end_count - 1) / 2 * spacing

  for ii in range(start_count) {
    for jj in range(end_count) {
      let start = (start_pos, start_y - ii * spacing)
      let end = (end_pos, end_y - jj * spacing)
      draw.line(start, end, stroke: base3 + 1pt)
    }
  }
}

#let main() = {
  set text(fill: base6, size: 16pt)
  // [#layers]

  canvas(padding: (y: 2pt), {
    // Zero-size markers, kanva collects all DrawPath commands between them.
    content((0, 0), [#box()<connections-start>])
    for idx in range(layers.len() - 1) {
      let (x1, n1, ..) = layers.at(idx)
      let (x2, n2, ..) = layers.at(idx + 1)
      connect_layers(x1, n1, x2, n2)
    }
    content((0, 0), [#box()<connections-end>])

    content((layers.at(0).at(0), 5.2), align(center)[Input Layer])
    content((layers.at(2).at(0), 4.2), align(center)[Latent\ Representation])
    content((layers.at(-1).at(0), 5.2), align(center)[Output Layer])

    for (x, count, fill, prefix, sup) in layers {
      let y_offset = (count - 1) / 2 * spacing
      for idx in range(count) {
        let y_pos = y_offset - idx * spacing
        let label = if sup != none {
          $prefix^sup_idx$
        } else if prefix == "hat(x)" {
          $hat(x)_idx$
        } else {
          $prefix_idx$
        }
        neuron((x, y_pos), fill: fill, text: label)
      }
    }
  })
}
