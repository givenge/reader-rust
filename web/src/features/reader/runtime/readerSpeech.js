export default {
  data() {
    return {
      speechAvalable: false,
      showReadBar: false,
      voiceList: [],
      speechSpeaking: false,
      showSpeechConfig: true,
      speechMinutes: 0,
      speechEndTime: 0,
      isSpeechTransitioning: false,
      skipAutoNext: false,
      utterance: null
    };
  },
  methods: {
    fetchVoiceList() {
      this.voiceList = window.speechSynthesis.getVoices().sort((a, b) => {
        if (a.lang.startsWith("zh-") && b.lang.startsWith("zh-")) {
          return a.lang > b.lang ? 1 : a.lang < b.lang ? -1 : 0;
        }
        if (a.lang.startsWith("zh-")) {
          return -1;
        }
        if (b.lang.startsWith("zh-")) {
          return 1;
        }
        return a.lang > b.lang ? 1 : a.lang < b.lang ? -1 : 0;
      });
      if (!this.voiceName && this.voiceList.length > 0) {
        const zhVoice = this.voiceList.find(v => v.lang.startsWith("zh-"));
        const defaultVoice = zhVoice || this.voiceList[0];
        this.$store.commit("setSpeechVoiceConfig", {
          ...this.$store.state.speechVoiceConfig,
          voiceName: defaultVoice.name
        });
      }
    },
    changeSpeechRate(rate) {
      this.speechRate = rate;
    },
    changeSpeechPitch(pitch) {
      this.speechPitch = pitch;
    },
    changeSpeechMinutes(minute) {
      this.speechMinutes = minute;
      if (minute) {
        this.speechEndTime = new Date().getTime() + minute * 60 * 1000;
      } else {
        this.speechEndTime = 0;
      }
    },
    startSpeech() {
      if (this.error) {
        return;
      }
      if (!this.voiceList.length) {
        this.fetchVoiceList();
      }
      if (!this.voiceName) {
        this.$message.warning("请先选择语音库");
        return;
      }
      const voice = this.voiceList.find(v => v.name === this.voiceName);
      if (!voice) {
        this.$message.warning("未找到所选语音，请重新选择");
        return;
      }
      if (window.speechSynthesis.speaking) {
        this.stopSpeech();
        setTimeout(() => {
          this.startSpeech();
        }, 100);
        return;
      }
      if (
        this.speechSpeaking &&
        this.speechEndTime > 0 &&
        new Date().getTime() > this.speechEndTime
      ) {
        this.$message.info("定时关闭朗读");
        return;
      }
      const paragraph = this.getCurrentParagraph();
      if (!paragraph || !paragraph.innerText) {
        this.speechNext();
        return;
      }
      this.utterance = new SpeechSynthesisUtterance(paragraph.innerText);
      this.utterance.onstart = () => {
        this.speechSpeaking = true;
        this.skipAutoNext = false;
      };
      this.utterance.onend = () => {
        if (!this.skipAutoNext) {
          this.speechSpeaking = false;
          this.speechNext();
        } else {
          this.skipAutoNext = false;
          this.speechSpeaking = false;
        }
      };
      this.utterance.onerror = event => {
        if (
          event.error === "interrupted" ||
          event.error === "canceled"
        ) {
          return;
        }
        if (event.error || event.name) {
          this.$message.error(
            `朗读错误:  ${event.type || ""}  ${event.error ||
              event.name ||
              event.toString()}`
          );
        }
        this.speechSpeaking = false;
      };
      this.utterance.voice = voice;
      this.utterance.pitch = this.speechPitch;
      this.utterance.rate = this.speechRate;
      this.showParagraph(paragraph, true);
      paragraph.className = "reading";
      this.speechSpeaking = true;
      window.speechSynthesis.speak(this.utterance);
    },
    stopSpeech(clearReadingClass = true) {
      try {
        this.skipAutoNext = true;
        this.speechSpeaking = false;
        window.speechSynthesis.cancel();
        if (clearReadingClass) {
          const current = this.getCurrentParagraph();
          if (current) {
            current.className = "";
          }
        }
      } catch (error) {
        //
      }
    },
    restartSpeech() {
      this.stopSpeech(false);
      setTimeout(() => {
        this.startSpeech();
      }, 150);
    },
    toggleSpeech() {
      this.speechSpeaking ? this.stopSpeech() : this.startSpeech();
    },
    speechPrev() {
      const current = this.getCurrentParagraph();
      const prev = this.getPrevParagraph();
      if (window.speechSynthesis.speaking) {
        this.stopSpeech(false);
      }
      if (prev) {
        if (current) {
          current.className = "";
        }
        prev.className = "reading";
        this.showParagraph(prev, true);
        setTimeout(() => {
          this.startSpeech();
        }, 150);
      } else {
        if (current) {
          current.className = "";
        }
        this.$once("showContent", () => {
          setTimeout(() => {
            this.startSpeech();
          }, 100);
        });
        this.toLastChapter();
      }
    },
    speechNext() {
      const current = this.getCurrentParagraph();
      const next = this.getNextParagraph();
      if (window.speechSynthesis.speaking) {
        this.stopSpeech(false);
      }
      if (next) {
        if (current) {
          current.className = "";
        }
        next.className = "reading";
        this.showParagraph(next, true);
        setTimeout(() => {
          this.startSpeech();
        }, 150);
      } else {
        if (current) {
          current.className = "";
        }
        this.$once("showContent", () => {
          setTimeout(() => {
            this.startSpeech();
          }, 100);
        });
        this.toNextChapter();
      }
    }
  }
};
