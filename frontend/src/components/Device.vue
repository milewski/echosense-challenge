<template>

    <div v-if="enabled" class="flex flex-col-reverse sm:flex-row justify-center items-center w-screen h-screen pt-8 bg-white">

        <VueDevice device="iphone-14" class="pt-8 bg-white">

            <div class="content">
                <slot/>
            </div>

        </VueDevice>

        <div class="flex flex-col items-start pt-8 bg-white text-gray-600 px-20 my-10">

            <div class="space-y-4 max-w-md">

                <img src="../assets/logo.png" alt="logo" class="w-[300px]">

                <div class="text-md">
                    <b>This is a running in simulation mode</b>.
                    <br>
                    <br>
                    <p>
                        When paired with the real device, the frontend establishes a WebSocket
                        connection to the device, where all AI processing and logic take place. In simulation mode, the
                        frontend streams only placeholder lorem ipsum text, mimicking the APIs and data the device would
                        otherwise send. To see the device in action,
                        <Button variant="link" size="lg" class="p-0 font-bold" as="a" target="_blank" href="https://dev.to/milewski/echosense-your-pocket-sized-companion-for-smarter-meetings-3i71">
                            refer to this post for more details.
                        </Button>
                    </p>
                </div>

            </div>


        </div>

    </div>

    <div v-else class="w-screen">
        <slot/>
    </div>

</template>

<script setup lang="ts">

    import { VueDevice } from 'vue-devices'
    import { boolean } from 'zod'
    import { Button } from '../../@/components/ui/button'

    defineProps({ enabled: boolean })

</script>

<style>

    #app .device div.content * {
        display: revert;
    }

</style>