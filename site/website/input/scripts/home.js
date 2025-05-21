import init, { Cube } from '/scripts/cube/spinning_square.js';
  async function run() {
      await init();

      const canvas = document.getElementById('canvas');
      const ctx = canvas.getContext('2d');

      const cube = Cube.new();

      function animate() {
          var color = getComputedStyle(document.documentElement).getPropertyValue('--text-color').trim();
          cube.update();
          cube.render(ctx, canvas.width, canvas.height, color);
          requestAnimationFrame(animate);
      }
      animate();
  }
  run();
