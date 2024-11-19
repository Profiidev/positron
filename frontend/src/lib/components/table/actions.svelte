<script lang="ts">
  import { Pencil, Trash } from "lucide-svelte";
  import { Button } from "../ui/button";

  interface Props {
    edit_disabled: boolean;
    delete_disabled: boolean;
    edit: () => void;
    remove: () => void;
  }

  let { edit_disabled, delete_disabled, edit, remove }: Props = $props();
</script>

<div class="flex space-x-4">
  <Button
    size="icon"
    variant="secondary"
    onclick={edit}
    disabled={edit_disabled}
  >
    <Pencil />
  </Button>
  <Button
    size="icon"
    variant="destructive"
    onclick={remove}
    disabled={delete_disabled}
  >
    <Trash />
  </Button>
</div>
