pub const SESSION: &str = r##"
    <script>
function setCookie(name,value,days) {
    var expires = "";
    if (days) {
        var date = new Date();
        date.setTime(date.getTime() + (days*24*60*60*1000));
        expires = "; expires=" + date.toUTCString();
    }
    document.cookie = name + "=" + (value || "")  + expires + "; path=/";
}
function getCookie(name) {
    var nameEQ = name + "=";
    var ca = document.cookie.split(';');
    for(var i=0;i < ca.length;i++) {
        var c = ca[i];
        while (c.charAt(0)==' ') c = c.substring(1,c.length);
        if (c.indexOf(nameEQ) == 0) return c.substring(nameEQ.length,c.length);
    }
    return null;
}
      $(document).ready(function () {
        const cookie = getCookie('passphrase');

        if (cookie) {
          $('#passphrase').val(cookie);
        }
        $('#start-session').click(function () {
          const passphrase = $('#passphrase').val();
          const boardId = document.getElementById("board-id").innerHTML;
          $.ajax({
              url: '/' + boardId + '/session',
              type: 'PUT',
              beforeSend: function(request) {
                request.setRequestHeader("passphrase", passphrase);
              },
              data: JSON.stringify({
                "players": $('#player-select').val().map(x => Number(x)),
                "game_info": {
                  "games": Number($("#num-games-select").val()),
                  "players_per_team": Number($("#players-per-select").val()),
                  "stability": Number($("#stability-select").val())
                }
              }),
          }).done(function() {
            setCookie("passphrase", passphrase)
            location.reload();
          });
        }); 
        $('#add-match').click(function () {
          const boardId = document.getElementById("board-id").innerHTML;
          const passphrase = $('#passphrase').val();
          $.ajax({
              url: '/' + boardId + '/add-match',
              type: 'PUT',
              beforeSend: function(request) {
                request.setRequestHeader("passphrase", passphrase);
              },
              data: JSON.stringify({
                "id": 0,
                "team1": $('#winners-select').val().map(x => Number(x)),
                "team2": $('#losers-select').val().map(x => Number(x)),
              }),
            }).done(function () {
                const successAddMatchToast = document.getElementById('add-match-toast');
                const toast = bootstrap.Toast.getOrCreateInstance(successAddMatchToast);
                toast.show();

                setCookie("passphrase", passphrase)
            });
          });
        $('#generate-matches').click(function () {
          const boardId = document.getElementById("board-id").innerHTML;
          const passphrase = $('#passphrase').val();
          $.ajax({
              url: '/' + boardId + '/generate-matches',
              type: 'PUT',
              beforeSend: function(request) {
                request.setRequestHeader("passphrase", passphrase);
              },
              data: JSON.stringify($('#matchmake-select').val().map(x => Number(x))),
              success: function (data) {
                document.getElementById("match-area").innerHTML = data;
              },
          });
        });
        $('#add-session').click(function () {
          const boardId = document.getElementById("board-id").innerHTML;
          const passphrase = $('#passphrase').val();
          $.ajax({
              url: '/' + boardId + '/session',
              type: 'PATCH',
              beforeSend: function(request) {
                request.setRequestHeader("passphrase", passphrase);
              },
              data: JSON.stringify($('#session-select').val().map(x => Number(x))),
          }).done(function() {
            setCookie("passphrase", passphrase)
            location.reload();
          });
        });
        $('#stop-session').click(function () {
          const boardId = document.getElementById("board-id").innerHTML;
          const passphrase = $('#passphrase').val();
          $.ajax({
              url: '/' + boardId + '/session',
              type: 'DELETE',
              beforeSend: function(request) {
                request.setRequestHeader("passphrase", passphrase);
              },
          }).done(function() {
            setCookie("passphrase", passphrase)
            location.reload();
          });
        });
      });
    </script>
  </body>
</html>
    "##;

pub const PLAYER: &str = r##"
<script>
function setCookie(name,value,days) {
    var expires = "";
    if (days) {
        var date = new Date();
        date.setTime(date.getTime() + (days*24*60*60*1000));
        expires = "; expires=" + date.toUTCString();
    }
    document.cookie = name + "=" + (value || "")  + expires + "; path=/";
}
function getCookie(name) {
    var nameEQ = name + "=";
    var ca = document.cookie.split(';');
    for(var i=0;i < ca.length;i++) {
        var c = ca[i];
        while (c.charAt(0)==' ') c = c.substring(1,c.length);
        if (c.indexOf(nameEQ) == 0) return c.substring(nameEQ.length,c.length);
    }
    return null;
}

      $(document).ready(function () {
        const cookie = getCookie('passphrase'); 
        if (cookie) {
          $('#passphrase').val(cookie);
        }

        $('#add-player').click(function () {
          const score = $("#player-score").val();
          const boardId = document.getElementById("board-id").innerHTML;
          const passphrase = $('#passphrase').val();

          $.ajax({
              url: '/' + boardId + '/players',
              type: 'POST',
              beforeSend: function(request) {
                request.setRequestHeader("passphrase", passphrase);
              },
              data: JSON.stringify({
                "name": $("#player-name").val(),
                "score": score ? Number(score) : null,
              }),
          }).done(function () {
            const successAddPlayerToast = document.getElementById('add-player-toast');
            const toast = bootstrap.Toast.getOrCreateInstance(successAddPlayerToast);
            toast.show();

            setCookie("passphrase", passphrase)
          });
        });
      });
    </script>
  </body>
</html>
"##;

pub const INDEX: &str = r##"
  <script>
      $(document).ready(function () {
        const boardId = document.getElementById("board-id").innerHTML;
        $('#players').click(function () {
           location.href = '/'+ boardId +'/player'
        });
        $('#session').click(function () {
           location.href = '/'+ boardId +'/sesh'
        });
      });
  </script>
  </body>
</html>
"##;
