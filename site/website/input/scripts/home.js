document.addEventListener('DOMContentLoaded', function () {
    const icons = document.querySelectorAll('.icon');
    let mainIcon = document.querySelector('.main-icon');
    
    icons.forEach(icon => {
        icon.addEventListener('click', function () {
            if (this === mainIcon) return;
            
            icons.forEach(i => {
                i.classList.remove('main-icon');
            });
            
            this.classList.add('main-icon');
            
            mainIcon = this;
            
        });
    });
});


import init, { SpinningCube } from '/scripts/cube/spinning_square.js';

  async function run() {
      await init();

      const canvas = document.getElementById('canvas');
      const ctx = canvas.getContext('2d');

      const cube = SpinningCube.new();

      function animate() {
          cube.update();
          cube.render(ctx, canvas.width, canvas.height);
          requestAnimationFrame(animate);
      }
      animate();
  }
  run();
