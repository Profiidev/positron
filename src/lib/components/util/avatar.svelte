<script lang="ts">
  import * as Avatar from "../ui/avatar";

  let { src, class: className }: { src: string; class: string } = $props();
</script>

<Avatar.Root class={className}>
  <Avatar.Image src={`data:image/webp;base64, ${src}`} />
  <Avatar.Fallback>?</Avatar.Fallback>
</Avatar.Root>
