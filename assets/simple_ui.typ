#import "monokai_pro.typ": *

#set page(
  height: auto,
  width: auto,
  fill: base0,
)
#let icon_config = (
  size: 24pt,
)
#(icon_config.padding = icon_config.size / 4)
#(icon_config.inset = icon_config.padding / 2)
#(icon_config.frame = icon_config.size + icon_config.padding)

#let font_config = (
  size: 16pt,
)
#(font_config.padding = font_config.size / 4)
#(font_config.inset = font_config.padding / 2)
#(font_config.frame = font_config.size + font_config.padding)

#let gradient_title(body) = {
  text(size: font_config.size * 2, fill: gradient.linear(blue, green))[= #body]
}

#let frame(body) = {
  set text(size: font_config.size, fill: base8)
  box(
    body,
    inset: font_config.inset * 2,
    radius: font_config.padding,
    fill: base1,
  )
}

#let button(body) = {
  set text(size: font_config.size, fill: base8)
  box(
    body,
    inset: font_config.inset * 2,
    radius: font_config.padding,
    fill: base2,
    stroke: 2pt + base6,
  )
}

#let icon(e) = {
  set text(size: icon_config.size, fill: base8)
  box(
    align(center + horizon)[#e],
    width: icon_config.frame,
    height: icon_config.frame,
    radius: icon_config.padding,
    inset: icon_config.inset,
    fill: base3,
  )
}

#let menu_item(top_content, bottom_content, height: 10em, width: 8em) = {
  set text(size: font_config.size, fill: base8)

  let top_height = height * 0.3
  let bottom_height = height * 0.7

  let half_height = height / 2
  let half_width = width / 2

  show par: set block(spacing: 0em)

  pad(1em)[
    #box(
      stroke: rgb(base7) + 0.15em,
      radius: 0.6em,
      clip: true,
    )[
      #align(center + horizon)[
        #box(
          width: width,
          height: top_height,
          fill: rgb(base3),
          inset: 0.5em,
        )[#top_content]
        #line(length: width, stroke: rgb(base5) + 4pt)
        #box(
          width: width,
          height: bottom_height,
          fill: rgb(base2),
          inset: 0.5em,
        )[#bottom_content]
      ]
    ]
  ]
}

#block(width: 1280pt, height: 720pt, inset: font_config.inset)[
  #gradient_title("Typst")
  #linebreak()
  #frame[
    #icon(emoji.clock.two)
    #icon(emoji.cloud)
    #icon(emoji.notebook)
  ]

  #menu_item([= Test], [#lorem(10)])
]
