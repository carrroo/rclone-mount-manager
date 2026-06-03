/**
 * Application entry point.
 * Sets up Vue with Pinia (state), vue-i18n (localization),
 * and global CSS tokens / base styles.
 */
import { createApp } from "vue";
import { createPinia } from "pinia";
import i18n from "./locales";
import App from "./App.vue";
import "./styles/base.css";
import "./styles/buttons.css";

const app = createApp(App);
app.use(createPinia());
app.use(i18n);
app.mount("#app");
