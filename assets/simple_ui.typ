#import "monokai_pro.typ": *

#set page(
  height: auto,
  width: auto,
  fill: base0,
)
#let icon_config = (
  size: 48pt,
)
#(icon_config.padding = icon_config.size / 4)
#(icon_config.inset = icon_config.padding / 2)
#(icon_config.frame = icon_config.size + icon_config.padding)

#let font_config = (
  size: 32pt,
)
#(font_config.padding = font_config.size / 4)
#(font_config.inset = font_config.padding / 2)
#(font_config.frame = font_config.size + font_config.padding)

#set text(size: font_config.size, fill: base8)

#let frame(body) = {
  box(
    body,
    inset: icon_config.inset * 2,
    radius: icon_config.padding,
    fill: base1,
  )
}

#let button(body) = {
  box(
    body,
    inset: icon_config.inset * 2,
    radius: icon_config.padding,
    fill: base2,
    stroke: 2pt + base6,
  )
}

#let icon(e) = {
  set text(size: icon_config.size)
  box(
    align(center + horizon)[#e],
    width: icon_config.frame,
    height: icon_config.frame,
    radius: icon_config.padding,
    inset: icon_config.inset,
    fill: base3,
  )
}

#block(width: 1280pt, height: 720pt, inset: font_config.inset)[
  #text(fill: gradient.linear(blue, green))[= Smart Assist]
  #frame[
    #icon(emoji.clock.two)
    #icon(emoji.cloud)
    #icon(emoji.notebook)
  ]

  #let size = 400pt
  #let half_size = size / 2
  #let offset = 50pt
  #for value in (1, 2, 3) {
    let o = offset * value
    place(
      top,
      dx: 40% - half_size + o - offset,
      dy: 50% - half_size + o - offset,
    )[
      #box(
        width: size,
        height: size,
        fill: rgb(base7.transparentize(60%)),
      )
    ]
  }

  #place(top + right)[
    #box(
      width: 360pt,
      height: 100%,
      radius: icon_config.padding,
      fill: gradient.linear(
        blue.transparentize(80%),
        red.transparentize(80%),
        angle: 15deg,
      ),
      stroke: gradient.linear(
        blue,
        red,
        angle: 15deg,
      ) + font_config.inset,
    )[
      #align(center + horizon)[ === Properties ]
    ]
  ]
]
