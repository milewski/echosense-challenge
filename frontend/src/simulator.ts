import { Payload } from './App.vue'
import { calculateElapsedTime } from './utilities.ts'
import { LoremIpsum } from 'lorem-ipsum'

const lorem = new LoremIpsum({
    sentencesPerParagraph: {
        max: 8,
        min: 4,
    },
    wordsPerSentence: {
        max: 16,
        min: 4,
    },
})

function randomInterval(): number {
    return Math.floor(Math.random() * 200)
}

function randomBetween(min: number, max: number): number {
    return Math.floor(Math.random() * (max - min)) + min
}

export function simulateSummary(onMessage: (message: Partial<Payload>) => void) {

    setTimeout(async () => {

        onMessage({
            Summary: Array.from({ length: randomBetween(5, 10) })
                .map(() => `• ${ lorem.generateSentences(randomBetween(2, 5)) }`)
                .join('\n'),
        })

    }, randomInterval() * 10)

}

export function simulateIntelligence(id: string, onMessage: (message: Partial<Payload>) => void) {

    setTimeout(async () => {

        onMessage({
            AnswerQuestion: {
                id,
                answer: lorem.generateSentences(5),
            },
        })

    }, randomInterval() * 10)

}

export function simulateLiveTranscription(onMessage: (message: Partial<Payload>) => void) {

    setTimeout(async () => {

        while (true) {

            const words: string[] = []

            for (const word of lorem.generateSentences(randomBetween(2, 10)).split(' ')) {

                await new Promise(resolve => {

                    words.push(word)

                    onMessage({
                        PartialTranscription: {
                            text: words.join(' '),
                            timestamp: new Date().toISOString(),
                        },
                    })

                    setTimeout(() => resolve(true), randomInterval())

                })

            }

            onMessage({
                FinalTranscription: {
                    text: words.join(' '),
                    timestamp: calculateElapsedTime((new Date).toISOString()),
                },
            })

        }

    })

}