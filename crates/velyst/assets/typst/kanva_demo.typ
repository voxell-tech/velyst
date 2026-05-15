#import "styles/monokai_pro.typ": *

#let main() = [
  #set text(size: 72pt, fill: orange)
  #set par(leading: 0.2em)
  #box(
    stroke: (paint: gradient.linear(purple, yellow), thickness: 2pt),
    width: 4em,
    height: 1.5em,
  )[
    #set align(center + horizon)
    #box[#text(weight: "bold", fill: yellow)[k]] <letter_k>
    #box[#text[anva]] <wordmark>
    #box[
      #line(length: 80%, stroke: (
        paint: orange,
        thickness: 2pt,
      ))
    ] <accent>
  ] <frame>
]
