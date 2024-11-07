<script lang="ts">
  import { goto } from "$app/navigation";

  goto("/account/profile", {
    replaceState: true,
  });
</script>
