// The only file that imports three.js.
import {
  Box3,
  BufferAttribute,
  BufferGeometry,
  Color,
  DirectionalLight,
  Group,
  HemisphereLight,
  Mesh,
  MeshStandardMaterial,
  PerspectiveCamera,
  SRGBColorSpace,
  Scene,
  Vector3,
  WebGLRenderer,
} from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import type { MeshBuffers, Rgb } from '../engine';
import type { MeshViewer } from './mesh-viewer';

/** Where the camera looks from, relative to the model: in front, above (z up). */
const VIEW_DIRECTION = new Vector3(0, -1, 1.1).normalize();

function colour({ r, g, b }: Rgb): Color {
  return new Color().setRGB(r / 255, g / 255, b / 255, SRGBColorSpace);
}

/**
 * A three.js viewer with orbit controls. It renders only when something
 * changes (the model, the camera or the size), not every frame.
 */
export function createThreeMeshViewer(container: HTMLElement): MeshViewer {
  const renderer = new WebGLRenderer({ antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.domElement.style.display = 'block';
  container.append(renderer.domElement);

  const scene = new Scene();
  const sky = new HemisphereLight(0xffffff, 0x60606a, 1.6);
  sky.position.set(0, 0, 1);
  const sun = new DirectionalLight(0xffffff, 1.6);
  sun.position.set(0.6, -1, 1.4);
  const model = new Group();
  scene.add(sky, sun, model);

  const camera = new PerspectiveCamera(35, 1, 0.1, 10_000);
  camera.up.set(0, 0, 1);
  const controls = new OrbitControls(camera, renderer.domElement);

  let meshes: Mesh<BufferGeometry, MeshStandardMaterial>[] = [];
  let fitted = false;

  const render = (): void => renderer.render(scene, camera);
  controls.addEventListener('change', render);

  const resize = (): void => {
    const { clientWidth: width, clientHeight: height } = container;
    if (width === 0 || height === 0) return;
    renderer.setSize(width, height);
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    render();
  };
  const observer = new ResizeObserver(resize);
  observer.observe(container);

  function clear(): void {
    for (const mesh of meshes) {
      mesh.geometry.dispose();
      mesh.material.dispose();
    }
    model.clear();
    meshes = [];
  }

  /** Frames the whole model, seen from the front and above. */
  function fit(): void {
    const box = new Box3().setFromObject(model);
    const centre = box.getCenter(new Vector3());
    const radius = box.getSize(new Vector3()).length() / 2;
    const distance = radius / Math.sin((camera.fov * Math.PI) / 360);
    camera.position.copy(centre).addScaledVector(VIEW_DIRECTION, distance);
    camera.near = distance / 100;
    camera.far = distance * 10;
    camera.updateProjectionMatrix();
    controls.target.copy(centre);
    controls.update();
  }

  return {
    show(mesh: MeshBuffers): void {
      clear();
      for (const part of mesh.parts) {
        const geometry = new BufferGeometry();
        geometry.setAttribute('position', new BufferAttribute(part.positions, 3));
        geometry.setIndex(new BufferAttribute(part.indices, 1));
        // Flat shading takes each triangle's own normal, so the walls and
        // tops stay crisp without normals from the engine.
        const material = new MeshStandardMaterial({
          color: colour(part.display),
          flatShading: true,
          roughness: 0.85,
        });
        const shape = new Mesh(geometry, material);
        meshes.push(shape);
        model.add(shape);
      }
      if (!fitted && meshes.length > 0) {
        fit();
        fitted = true;
      }
      resize();
      render();
    },

    dispose(): void {
      observer.disconnect();
      controls.removeEventListener('change', render);
      controls.dispose();
      clear();
      renderer.dispose();
      renderer.forceContextLoss();
      renderer.domElement.remove();
    },
  };
}
