import { createRoot } from 'react-dom/client';

window.onload = () => {
    // Render your React component instead
    const app = document.getElementById('app');
    if (app) {
        const root = createRoot(app);
        console.log("root created: ",root);
        root.render(<h1>Hello, world</h1>);
    }
}
