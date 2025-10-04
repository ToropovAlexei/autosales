import { createTheme } from '@mui/material/styles';
import type { ThemeOptions } from '@mui/material/styles';
import { colorSchemes, typography, shadows, shape } from './themePrimitives';

export const createAppTheme = (themeComponents?: ThemeOptions['components']) => {
  return createTheme({
    cssVariables: {
      colorSchemeSelector: 'data-mui-color-scheme',
      cssVarPrefix: 'template',
    },
    colorSchemes,
    typography,
    shadows,
    shape,
    components: {
      ...themeComponents,
    },
  });
};