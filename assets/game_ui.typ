#import "monokai_pro.typ": *

#set page(
  width: auto,
  height: auto,
  fill: black,
  margin: 0pt,
)

#let main(
  width,
  height,
  perf_metrics,
) = {
  set text(fill: base8)

  box(
    width: width,
    height: height,
    inset: (x: width * 6%, y: height * 6%),
  )[
    #place(left + horizon)[
      #set text(size: 48pt)
      #text(fill: yellow)[= Lumina]

      #linebreak()

      #move(dx: 2%, dy: -50pt)[
        #set text(size: 32pt, fill: base7)
        #text(fill: green)[= Play]
        = #text(fill: purple)[Luminators]
        = Tutorial
        = Watch #text(fill: green, size: 20pt)[
          #emoji.triangle.r 4152 Live Now
        ]

        #linebreak()

        #set text(size: 18pt, fill: red.transparentize(40%))
        #box()[= Exit Game] <btn:exit-game>
      ]
    ]

    #place(left + bottom)[
      #set text(size: 18pt)
      #emoji.gear Settings
    ]

    #let player_name = "Nixon"

    #place(right + top)[
      #set text(size: 18pt)

      #let size = 60pt
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
      #perf_metrics
    ]
  ]
}

#let perf_metrics(fps, elapsed_time) = {
  set text(size: 18pt)

  align(left)[
    = Performance Metrics
    FPS: #fps\
    Elapsed Time: #elapsed_time\
  ]
}

#main(1280pt, 720pt, perf_metrics(60, 1.23))
