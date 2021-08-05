const Main = {
  data() {
    return {
      auth_state: {
      },
      loading: true
    }
  },
  mounted() {
    console.log("Requesting authstate")
    axios.get('/api/authstate').then(response => {
      this.loading = false
      this.auth_state = response.data

      if (!this?.auth_state?.fitbit?.has_token) {
        const client_id = encodeURIComponent(this.auth_state.fitbit.client_id)
        const redirect_uri = encodeURIComponent(this.auth_state.fitbit.redirect_uri)
        const scopes = encodeURIComponent(this.auth_state.fitbit.scopes)
        window.location.href = `https://www.fitbit.com/oauth2/authorize?response_type=code&client_id=${client_id}&redirect_uri=${redirect_uri}&scope=${scopes}`
      }
    })
  },
  template: `
  <div><span>Fitbit: </span><status :ok="auth_state?.fitbit?.has_token" /></div>
  <div><span>Google: </span><status :ok="auth_state?.google?.has_token" /></div>
  `
}

const app = Vue.createApp(Main)

app.component('status', {
  props: ["ok"],
  template: `
    <span>
      <span class="OkStatus" v-if="ok">OK</span>
      <span class="NotOkStatus" v-else>Not OK</span>
    </span>
  `
})

app.mount('#app')