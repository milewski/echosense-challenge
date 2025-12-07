<template>

    <div class="space-x-2  transition-all duration-75 p-4 hover:bg-muted hover:border-l-4 border-black"
         :class="{ 'bg-muted border-l-4': index === transcription.length -1}">

        <Badge variant="outline" class="inline">07:11</Badge>

        <div class="inline">
            <slot/>
        </div>

    </div>

</template>

<script setup lang="ts">

    import { Badge } from '../../@/components/ui/badge'

</script>