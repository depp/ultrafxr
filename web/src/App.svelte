<script>
  import { playSound } from './audio.js';
</script>

<style>
  main {
    text-align: center;
    padding: 1em;
    max-width: 240px;
    margin: 0 auto;
  }

  @media (min-width: 640px) {
    main {
      max-width: none;
    }
  }
</style>

<main>
  <h1>UltraFXR</h1>
  <button on:click={playSound}>Play</button>
</main>
