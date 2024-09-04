#import "monokai_pro.typ": *

#set page(
  width: auto,
  height: auto,
  fill: black,
  margin: 0pt,
)

#let main(
  main_width: 1280pt,
  main_height: 720pt,
  fps: 60.0,
  elapsed_time: 1.32,
) = {
  set text(fill: base8)

  box(
    width: main_width,
    height: main_height,
    inset: (x: main_width * 6%, y: main_height * 6%),
  )[
    #place(left + horizon)[
      #set text(size: 48pt)
      #text(fill: yellow)[= Side Effects]

      #linebreak()

      #move(dx: 2%)[
        #set text(size: 32pt, fill: base7)
        #text(fill: green)[= Play]
        = #text(fill: purple)[Luminators]
        = Tutorial
        = Watch #text(fill: green, size: 20pt)[
          #emoji.triangle.r 4152 Live Now
        ]

        #linebreak()

        #set text(size: 18pt, fill: red.transparentize(40%))
        = Exit Game
      ]
    ]

    #place(left + bottom)[
      #set text(size: 18pt)
      #emoji.gear Settings
    ]

    #let player_name = "Nixon"

    #place(right + top)[
      #set text(size: 24pt, font: "Inter")

      #let size = 80pt
      #align(horizon)[
        #stack(
          dir: ltr,
          rect(fill: blue, width: size, height: size),
          box(
            width: 400pt,
            height: size,
            fill: base6.transparentize(80%),
            inset: 20pt,
          )[
            #stack(
              dir: ltr,
              spacing: 1fr,
              player_name,
              underline[View Profile],
            )
          ],
        )
      ]
    ]

    #place(right + bottom)[
      #set text(size: 18pt)

      #align(left)[
        = Performance Metrics
        FPS: #box()[#text(fill: gradient.linear(
            red,
            orange,
            yellow,
            green,
            blue,
            purple
        ))[#fps]]\
        Elapsed Time: #elapsed_time\
      ]
    ]

  ]
}

#main()
