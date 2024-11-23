<template>

    <Tabs default-value="transcription" class="w-screen p-2">

        <TabsList class="grid w-full grid-cols-3">

            <TabsTrigger value="transcription">
                Transcription
            </TabsTrigger>

            <TabsTrigger value="intelligence">
                Intelligence
            </TabsTrigger>

            <TabsTrigger value="summary">
                Summary
            </TabsTrigger>

        </TabsList>

        <TabsContent value="transcription">

            <Card>

                <CardHeader>

                    <CardTitle>
                        Live Transcription
                    </CardTitle>

                    <CardDescription>
                        Below is the live conversation currently in progress.
                    </CardDescription>

                </CardHeader>

                <CardContent class="space-y-2">

                    <div class="sticky top-10 z-10">

                        <Input v-model="search" type="text" placeholder="Search..." class="pl-10"/>

                        <div class="absolute start-0 inset-y-0 flex items-center justify-center px-2">
                            <Search class="size-6 text-muted-foreground"/>
                        </div>

                        <div v-if="search" class="absolute end-0 inset-y-0 flex items-center justify-center px-2">
                            <XIcon class="size-6 text-muted-foreground" @click="search = ''"/>
                        </div>

                    </div>

                    <ScrollArea class="h-[500px] rounded-md border" @pointerdown="onPointerDown">

                        <div v-for="{ text, timestamp } of filteredTranscriptions" ref="lines"
                             class="space-x-2 relative transition-all duration-75 p-4 hover:bg-muted border-l-4 -left-1 hover:left-0 hover:border-black">

                            <Badge variant="outline" class="inline">{{ timestamp }}</Badge>

                            <div class="inline" v-html="text"/>

                        </div>

                        <div class="bg-muted border-l-4 border-black flex items-center p-4 text-gray-500">
                            {{ partialTranscription }}
                        </div>

                        <div class="relative h-10 bottom-5" ref="scrollTarget"/>

                    </ScrollArea>

                    <Drawer>

                        <DrawerTrigger as-child>

                            <Button variant="default" size="lg" class="w-full">
                                Terminate
                            </Button>

                        </DrawerTrigger>

                        <DrawerContent>

                            <div class="mx-auto w-full max-w-2xl">

                                <DrawerHeader>

                                    <DrawerTitle>Would you like to email the transcription?</DrawerTitle>

                                    <DrawerDescription>
                                        Enter your email address to receive the <br> transcription in your inbox.
                                    </DrawerDescription>

                                </DrawerHeader>

                                <div class="p-4 pb-0 space-y-4">

                                    <Input
                                        v-model="email"
                                        type="text"
                                        placeholder="Enter your email address"/>

                                    <div class="flex items-center space-x-2">

                                        <Checkbox id="terms"/>

                                        <Label
                                            for="terms"
                                            class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">

                                            Attach the audio recording

                                        </Label>

                                    </div>


                                </div>

                                <DrawerFooter>

                                    <DrawerClose as-child>
                                        <Button @click="sendAndStopRecording">Send and Stop Recording</Button>
                                    </DrawerClose>

                                    <DrawerClose as-child>

                                        <Button variant="outline" @click="stopRecordingOnly">
                                            Stop Recording Only
                                        </Button>

                                    </DrawerClose>

                                </DrawerFooter>

                            </div>

                        </DrawerContent>

                    </Drawer>

                </CardContent>

            </Card>

        </TabsContent>

        <TabsContent value="intelligence" class="space-y-4">

            <Card>

                <CardHeader>

                    <CardTitle>Intelligence</CardTitle>

                    <CardDescription>
                        Interact with the live transcription by asking questions to uncover deeper insights or view it
                        from a fresh perspective.
                    </CardDescription>

                </CardHeader>

                <CardContent class="space-y-2">
                    <Textarea v-model="questionPrompt" placeholder="Type your question about the current stream..."/>
                </CardContent>

                <CardFooter>
                    <Button @click.capture="submitQuestion" :disabled="!questionPrompt">Submit Question</Button>
                </CardFooter>

            </Card>

            <Card v-if="accordionItems.length">

                <CardContent class="space-y-2">

                    <Accordion type="single" class="w-full" collapsible v-model="accordionState">

                        <AccordionItem
                            class="last:border-none"
                            v-for="item in accordionItems"
                            :key="item.id"
                            :value="item.id"
                            :class="{ 'pointer-events-none': item.loading }">

                            <AccordionTrigger class="flex">

                                <div class="text-left w-11/12">{{ item.title }}</div>

                                <template #icon v-if="item.loading">
                                    <Loader class="size-6 ml-auto mr-4 text-muted-foreground animate-spin"/>
                                </template>

                            </AccordionTrigger>

                            <AccordionContent>
                                {{ item.content }}
                            </AccordionContent>

                        </AccordionItem>

                    </Accordion>

                </CardContent>

            </Card>

        </TabsContent>

        <TabsContent value="summary">

            <Card>

                <CardHeader>

                    <CardTitle>Summary</CardTitle>

                    <CardDescription>
                        Review the summary and key points of the transcription so far...
                    </CardDescription>

                </CardHeader>

                <CardContent class="space-y-2">

                    <div v-if="summary.length">

                        <ul class="my-6 ml-6 list-disc [&>li]:mt-4">
                            <li v-for="line of summary">{{ line }}</li>
                        </ul>

                    </div>

                    <div v-else
                         class="flex p-5 w-full items-center justify-center rounded-md border border-dashed text-sm">
                        No summary generated yet
                    </div>

                </CardContent>

                <CardFooter>

                    <Button @click.capture="getSummary" :disabled="transcription.length === 0 || isSummaryLoading"
                            class="w-full">
                        <div>Get Summary</div>
                        <Loader v-if="isSummaryLoading" class="size-6 text-muted-foreground animate-spin"/>
                    </Button>

                </CardFooter>

            </Card>

        </TabsContent>

    </Tabs>

</template>

<script setup lang="ts">

    import { Button } from '../@/components/ui/button'
    import { Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerTitle, DrawerTrigger } from '../@/components/ui/drawer'
    import { Textarea } from '../@/components/ui/textarea'
    import { Tabs, TabsContent, TabsList, TabsTrigger } from '../@/components/ui/tabs'
    import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../@/components/ui/card'
    import { computed, onMounted, ref, useTemplateRef } from 'vue'
    import { ScrollArea } from '../@/components/ui/scroll-area'
    import { Badge } from '../@/components/ui/badge'
    import { Input } from '../@/components/ui/input'
    import { Loader, Search, XIcon } from 'lucide-vue-next'
    import { Checkbox } from '../@/components/ui/checkbox'
    import { Label } from '../@/components/ui/label'
    import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '../@/components/ui/accordion'
    import { simulateIntelligence, simulateLiveTranscription, simulateSummary } from './simulator.ts'
    import { calculateElapsedTime, guidGenerator } from './utilities.ts'

    const accordionItems = ref<Array<{ id: string, title: string, content: string | null, loading: boolean }>>([])
    const accordionState = ref()
    const transcription = ref<{ text: string, timestamp: string }[]>([])
    const search = ref('')
    const filteredTranscriptions = computed(() => {

        if (search.value) {

            return transcription.value
                .filter(transcription => transcription.text.toLowerCase().includes(search.value.toLowerCase()))
                .map(({ text, timestamp }) => ({
                    timestamp,
                    text: text.replace(
                        new RegExp(`${ search.value }`, 'gi'), `<mark><b>${ search.value }</b></mark>`,
                    ),
                }))

        }

        return transcription.value

    })

    const partialTranscription = ref('listening...')
    const summary = ref<string[]>([])
    const lines = useTemplateRef<Array<HTMLElement>>('lines')
    const scrollTarget = ref<HTMLElement>()
    const pointerDown = ref(false)
    const isInView = ref(true)
    const isSummaryLoading = ref(false)
    const questionPrompt = ref()
    const isSimulation = window.location.search.includes('simulation')
    const email = ref()

    export type Payload = {
        PartialTranscription?: { text: string, timestamp: string },
        FinalTranscription?: { text: string, timestamp: string },
        Transcriptions?: Array<{ text: string, timestamp: string }>,
        AnswerQuestion: { answer: string, id: string },
        Summary?: string,
        SessionId?: number,
    }

    const query = new URLSearchParams(window.location.search)
    const websocket_host = query.get('ws') ?? window.location.host
    const ws = new WebSocket(`ws://${ websocket_host }/connect`)

    if (isSimulation) {
        simulateLiveTranscription(onMessage)
    }

    function onPointerDown() {
        pointerDown.value = true
        setTimeout(() => pointerDown.value = false, 200)
    }

    ws.onopen = function (event: Event) {
        console.log('open', event)
    }

    ws.onclose = ws.onerror = function (event: Event) {
        console.log('close', event)
    }

    function submitQuestion() {

        const id = guidGenerator()

        const data = {
            id,
            loading: true,
            title: questionPrompt.value,
            content: null,
        }

        if (isSimulation) {

            simulateIntelligence(id, onMessage)

        } else {
            ws.send(JSON.stringify({ command: { AskQuestion: { id, question: questionPrompt.value } } }))
        }

        accordionItems.value.unshift(data)
        questionPrompt.value = ''

    }

    ws.onmessage = function (event: MessageEvent<string>) {

        const data: Payload = JSON.parse(event.data)
console.log(data)
        if (data.PartialTranscription) {
            data.PartialTranscription.timestamp = calculateElapsedTime(data.PartialTranscription.timestamp)
        }

        if (data.FinalTranscription) {
            data.FinalTranscription.timestamp = calculateElapsedTime(data.FinalTranscription.timestamp)
        }

        if (data.Transcriptions) {

            for (const transcription of data.Transcriptions) {
                transcription.timestamp = calculateElapsedTime(transcription.timestamp)
            }

        }

        onMessage(data)

    }

    onMounted(() => {

        const rootElement = scrollTarget.value!

        const observer = new IntersectionObserver(entries => {
            entries.forEach(entry => {
                isInView.value = entry.isIntersecting
            })
        }, { threshold: 0.1 })  // Trigger when 10% of the div is visible

        observer.observe(rootElement)

    })

    function stopRecordingOnly() {

        ws.send(JSON.stringify({
            command: {
                SendTranscriptionViaEmail: {
                    email: null,
                    with_audio: false,
                },
            },
        }))

        ws.close(0, 'manually closed.')

    }

    function sendAndStopRecording() {

        ws.send(JSON.stringify({
            command: {
                SendTranscriptionViaEmail: {
                    email: email.value,
                    with_audio: false,
                },
            },
        }))

        email.value = null;

        ws.close(0, 'manually closed.')

    }

    function getSummary() {

        isSummaryLoading.value = true

        if (isSimulation) {

            simulateSummary(onMessage)

        } else {
            ws.send(JSON.stringify({ command: 'GetSummary' }))
        }

    }

    function onMessage(message: Partial<Payload>) {

        if (lines.value) {

            if (scrollTarget.value && !search.value) {

                setTimeout(() => {

                    const rootElement = scrollTarget.value?.parentElement?.parentElement

                    if (rootElement) {

                        if (isInView.value && pointerDown.value === false) {

                            rootElement.scrollBy({
                                top: rootElement.scrollHeight,
                                behavior: 'smooth',
                            })

                        }

                    }

                })

            }

        }

        if (message.Summary) {

            summary.value = message.Summary
                .split('\n')
                .map(element => element.replace(/[• ]/, ''))

            isSummaryLoading.value = false
        }

        if (message.Transcriptions) {

            for (const { text, timestamp } of message.Transcriptions) {

                transcription.value.push({
                    timestamp: timestamp,
                    text: text,
                })

            }

        }

        if (message.PartialTranscription) {
            partialTranscription.value = message.PartialTranscription.text
        }

        if (message.FinalTranscription) {

            transcription.value.push({
                timestamp: message.FinalTranscription.timestamp,
                text: message.FinalTranscription.text,
            })

            partialTranscription.value = 'listening...'

        }

        if (message.AnswerQuestion) {

            const item = accordionItems.value.find(element => element.id === message.AnswerQuestion!.id)

            if (item) {
                item.loading = false
                item.content = message.AnswerQuestion.answer
                accordionState.value = item.id
            }

        }
    }

</script>