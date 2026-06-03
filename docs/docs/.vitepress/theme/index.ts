import DefaultTheme from "vitepress/theme";
import Layout from "./Layout.vue";
import ModList from "./ModList.vue";
import "./custom.css";

export default {
  extends: DefaultTheme,
  Layout,
  enhanceApp({ app }) {
    app.component("ModList", ModList);
  },
};
