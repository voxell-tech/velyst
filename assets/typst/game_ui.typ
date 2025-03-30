#import "styles/monokai_pro.typ": *

#set page(
    width: auto,
    height: auto,
    fill: black,
    margin: 0pt,
)

#let button(body, lbl: label) = {
    [#box(inset: 16pt)[#body] #lbl]
}

#let main(
    width,
    height,
    perf_metrics,
    btn_highlight: "",
    animate: 0.0,
) = {
    set text(fill: base8)
    show label(btn_highlight): body => [
        #let box_fill = text.fill.transparentize(((1.0 - animate) * 100%))
        #set text(
            fill: color.mix(
                (text.fill, ((1.0 - animate) * 100%)),
                (base0, animate * 100%),
            ),
        )
        #box(fill: box_fill, radius: 10pt, outset: (animate * 6pt))[#body]
    ]

    let width = (width * 1pt)
    let height = (height * 1pt)

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
                #set text(size: 28pt, fill: base7)
                #text(fill: blue)[#button(lbl: <btn:play>)[= Play]]\
                #text(fill: purple)[#button(
                        lbl: <btn:luminators>,
                    )[= Luminators]]\
                #button(lbl: <btn:tutorial>)[= Tutorial]\
                #stack(
                    dir: ltr,
                    spacing: 10pt,
                    text(fill: green)[#button(lbl: <btn:watch>)[= Watch]],
                    text(fill: green, size: 16pt)[
                        #emoji.triangle.r 4152 Live Now
                    ],
                )

                #set text(size: 16pt, fill: red.transparentize(40%))
                #button(lbl: <btn:exit-game>)[= Exit Game]
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
                        width: 300pt,
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
    set text(size: 12pt)

    box(fill: base0.transparentize(60%), outset: 15pt, radius: 4pt)[
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
}

#main(1280, 720, perf_metrics(60, 1.23))
