/* 
 * Adapted from https://tonybox.net/posts/simple-stl-viewer/
 */
import * as THREE from './three.module.min.js';
import { OrbitControls } from './OrbitControls.js';
import { STLLoader } from './STLLoader.js';

function STLViewer(model, elem) {
  var camera = new THREE.PerspectiveCamera(70, 
    elem.clientWidth / elem.clientHeight, 1, 1000);

  var renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
  renderer.setSize(elem.clientWidth, elem.clientHeight);
  elem.appendChild(renderer.domElement);

  window.addEventListener('resize', function () {
    renderer.setSize(elem.clientWidth, elem.clientHeight);
    camera.aspect = elem.clientWidth / elem.clientHeight;
    camera.updateProjectionMatrix();
  }, false);

  var controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.rotateSpeed = 0.25;
  controls.dampingFactor = 0.1;
  controls.enableZoom = true;
  controls.autoRotate = true;
  controls.autoRotateSpeed = 2;

  var scene = new THREE.Scene();
  scene.add(new THREE.HemisphereLight(0xffffff, 1.5));

  (new STLLoader()).load(model, function (geometry) {
    var material = new THREE.MeshPhongMaterial({ 
        //color: 0xff5533, 
        color: 0xb18ed8, 
        specular: 100, 
        shininess: 80 });
    var mesh = new THREE.Mesh(geometry, material);
    scene.add(mesh);

    var middle = new THREE.Vector3();
    geometry.computeBoundingBox();
    geometry.boundingBox.getCenter(middle);
    mesh.geometry.applyMatrix4(new THREE.Matrix4().makeTranslation( 
                                  -middle.x, -middle.y, -middle.z ));

    var largestDimension = Math.max(geometry.boundingBox.max.x,
                                    geometry.boundingBox.max.y, 
                                    geometry.boundingBox.max.z);
    camera.position.z = largestDimension * 2.5;

    var animate = function () {
      requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    }; 

    animate();
  });
}



document.addEventListener('DOMContentLoaded', () => {
  const elements = document.querySelectorAll('.stl-view');
  elements.forEach((el, i) => {
    const stl = el.dataset.file;
    STLViewer(stl, el);
  });
});
