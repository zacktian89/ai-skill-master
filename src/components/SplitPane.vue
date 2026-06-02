<template>
  <div class="split-content">
    <section class="list-panel">
      <slot name="left"></slot>
    </section>
    <section class="detail-panel">
      <slot name="right"></slot>
    </section>
  </div>
</template>
