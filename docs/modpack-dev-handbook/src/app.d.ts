import "unplugin-icons/types/svelte";

declare global {
  type SearchPage = {
    title: string;
    description?: string;
    content: string;
    url: string;
    tags: string[] | undefined;
  };

  namespace App {}
}

export {};
